fn main() {
    connectrpc_build::Config::new()
        .files(&["proto/mu/spike/v1/portal.proto"])
        .includes(&["proto/"])
        .include_file("_connectrpc.rs")
        .compile()
        .unwrap();
}
