// Zeta Reticula is a decentralized quantization aware AI model and store.
// pb is the protobuf definitions for the Zeta Reticula sidecar.

tonic::include_proto!("zeta");
tonic::include_proto!("policy");
tonic::include_proto!("kvquant");
tonic::include_proto!("nsrouter");
tonic::include_proto!("salience");
tonic::include_proto!("sidecar");

pub mod zeta {
    tonic::include_proto!("zeta");
}
pub mod policy {
    tonic::include_proto!("policy");
}
pub mod kvquant {
    tonic::include_proto!("kvquant");
}
pub mod nsrouter {
    tonic::include_proto!("nsrouter");
}
pub mod salience {
    tonic::include_proto!("salience");
}
pub mod sidecar {
    tonic::include_proto!("sidecar");
}


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Protobuf error: {0}")]
    ProtobufError(#[from] prost::EncodeError),
}


impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        Error::GrpcError(status)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError(error)
    }
}

impl From<prost::EncodeError> for Error {
    fn from(error: prost::EncodeError) -> Self {
        Error::ProtobufError(error)
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}