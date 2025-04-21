use std::env;

fn main() {
    let proto_dir_path = env::var("BASKET_PROTO_DIRECTORY_PATH")
        .expect("Missing BASKET_PROTO_DIRECTORY_PATH env variable");

    let proto_files = [
        "add_to_queue_request.proto",
        "hp_request.proto",
        "ups_request.proto",
    ];

    let proto_paths = proto_files.map(|proto_file| format!("{}/{}", proto_dir_path, proto_file));

    for proto_path in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto_path);
    }

    tonic_build::configure()
        .compile_protos(&proto_paths, &[proto_dir_path])
        .expect("Failed to compile proto files");
}
