use actix_web::{web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use chrono::Utc;
use stripe::{Client, CreateCheckoutSession, CheckoutSession};

#[derive(Deserialize)]
struct SubscriptionRequest {
    plan: String, // "basic", "pro", "enterprise"
    email: String,
}

#[derive(Serialize)]
struct SubscriptionResponse {
    status: String,
    subscription_id: String,
    expires_at: String,
    checkout_url: Option<String>, // For Stripe redirect
}

async fn subscribe(
    req: web::Json<SubscriptionRequest>,
    data: web::Data<AppState>,
) -> Result<impl Responder, actix_web::Error> {
    let stripe = &data.stripe_client;
    let subscription_id = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + chrono::Duration::days(30); // Default 30 days

    let price_id = match req.plan.as_str() {
        "pro" => "price_1ProPlan", // Replace with actual Stripe Price ID
        "enterprise" => "price_1EnterprisePlan", // Replace with actual Stripe Price ID
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid plan")),
    };

    let session = stripe.checkout.sessions.create(
        &CreateCheckoutSession {
            success_url: Some("https://zeta-reticula.vercel.app/success".to_string()),
            cancel_url: Some("https://zeta-reticula.vercel.app/cancel".to_string()),
            payment_method_types: vec!["card".to_string()],
            line_items: vec![stripe::CheckoutSessionLineItemParams {
                price: price_id.to_string(),
                quantity: Some(1),
            }],
            mode: stripe::CheckoutSessionMode::Subscription,
            customer_email: Some(req.email.clone()),
            ..Default::default()
        },
    ).await.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    sqlx::query("INSERT INTO subscriptions (id, email, plan, expires_at, stripe_session_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(&subscription_id)
        .bind(&req.email)
        .bind(&req.plan)
        .bind(expires_at)
        .bind(&session.id)
        .execute(&data.db)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(web::Json(SubscriptionResponse {
        status: "success".to_string(),
        subscription_id,
        expires_at: expires_at.to_rfc3339(),
        checkout_url: Some(session.url.unwrap_or_default()),
    }))
}

struct AppState {
    db: PgPool,
    stripe_client: Client,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_url = std::env::var("NEON_CONNECTION_STRING").expect("NEON_CONNECTION_STRING must be set");
    let db = PgPool::connect(&db_url).await.expect("Failed to connect to Neon");
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
    let stripe_client = Client::new(stripe_secret);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db: db.clone(),
                stripe_client: stripe_client.clone(),
            }))
            .service(web::resource("/subscribe").to(subscribe))
            .service(web::resource("/infer").to(inference_handler::infer))
            .service(web::resource("/quantize").to(quantization_handler::quantize))
            .service(web::resource("/feedback").to(feedback_handler))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}