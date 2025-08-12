fn main() {
    tonic_build::compile_protos("../zeta-sidecar/proto/sidecar.proto").unwrap();
}
