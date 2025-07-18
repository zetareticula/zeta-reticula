use llm_rs::fusion_anns::FusionANNS;
use ndarray::array;
use tokio;

#[tokio::main]
async fn main() {
    // Initialize the FusionANNS with vector dimension 128 and batch size 32
    let vector_dim = 128;
    let batch_size = 32;
    
    println!("Creating FusionANNS instance...");
    let mut fusion_anns = FusionANNS::new(vector_dim, batch_size);
    
    println!("Initializing FusionANNS...");
    fusion_anns.initialize().await;
    
    // Create a sample query vector
    let query = array![1.0; 128];
    
    println!("Running collaborative filtering...");
    let top_m = 10;
    let results = fusion_anns.collaborative_filter(&query, top_m);
    
    println!("Top {} results: {:?}", top_m, results);
    
    // Test heuristic reranking
    println!("\nTesting heuristic reranking...");
    let candidates = (0..20).collect::<Vec<u32>>();
    let reranked = fusion_anns.heuristic_rerank(&query, candidates);
    
    println!("Reranked results: {:?}", reranked);
}
