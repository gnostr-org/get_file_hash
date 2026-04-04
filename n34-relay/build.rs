//#[cfg(feature = "gen-protos")]
fn compile_protos() {
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .build_transport(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["proto/plugins.proto"], &["proto"])
        .expect("protoc is required");
}

//#[cfg(not(feature = "gen-protos"))]
//fn compile_protos() {}


fn main() {
    compile_protos();
}
