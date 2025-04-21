use std::{env, path::PathBuf};

fn main() {
    let proto_dir_path = PathBuf::from(
        env::var("BASKET_PROTO_DIRECTORY_PATH")
            .expect("Missing BASKET_PROTO_DIRECTORY_PATH env variable"),
    );

    // NOTE: use the paths relative to the include paths
    let proto_files = [
        "add_to_queue_request.proto",
        "basket_service_requests/hp_request.proto",
        "basket_service_requests/ups_request.proto",
        "basket_service.proto",
    ];

    let include_paths = [
        proto_dir_path.clone(),
        proto_dir_path.join("basket_service_requests"),
    ];

    // Watch for changes to proto files and dirs
    for proto_file in &proto_files {
        println!(
            "cargo:rerun-if-changed={}",
            proto_dir_path.join(proto_file).display()
        );
    }

    for include_path in &include_paths {
        println!("cargo:rerun-if-changed={}", include_path.display());
    }

    tonic_build::configure()
        .compile_protos(&proto_files, &include_paths)
        .expect("Failed to compile proto files");
}
