use prost::Message;

pub mod zeta {
    pub mod policy {
        tonic::include_proto!("zeta.policy");
    }
}
