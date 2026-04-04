fn main() {
    tonic_prost_build::configure()
        .build_server(true) // For the examples
        .build_client(true)
        .build_transport(true)
        .compile_protos(&["proto/plugins.proto"], &["proto"])
        .unwrap();
}
