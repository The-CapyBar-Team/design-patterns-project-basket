use std::env;

fn main() {
    let proto_dir_path = env::var("BASKET_PROTO_DIRECTORY_PATH").unwrap();
    let proto_files = [
        "add_to_queue_request.proto",
        "hp_request.proto",
        "ups_request.proto",
    ];
    let proto_paths = proto_files.map(|proto_file| format!("{}/{}", proto_dir_path, proto_file));

    for proto_path in proto_paths.iter() {
        println!("{}", format!("cargo:rerun-if-changed={}", proto_path));
    }

    prost_build::compile_protos(&proto_paths, &[proto_dir_path]).unwrap();
}
