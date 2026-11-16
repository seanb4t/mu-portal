//! Generates the code of the schema with `connectrpc-build`.

fn main() {
    connectrpc_build::Config::new()
        .files(&["proto/mu/v1/portal.proto"])
        .includes(&["proto/"])
        // Connect and gRPC carry binary protobuf. The JSON codec needs serde.
        .generate_json(false)
        .include_file("_connectrpc.rs")
        .compile()
        .expect("the schema compiles");
}
