fn main() {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/plugins.proto"], &["proto"])
        .expect("protoc is required to build n34-relay. Install protobuf or set PROTOC to the protoc binary path.");
}
