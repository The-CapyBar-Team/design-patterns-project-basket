use std::env;

fn main() {
    let proto_dir_path = env::var("BASKET_PROTO_DIRECTORY_PATH")
        .expect("Missing BASKET_PROTO_DIRECTORY_PATH env variable");

    let proto_files = [
        "basket_service_requests/hp_request.proto",
        "basket_service_requests/ups_request.proto",
        "basket_service_requests/ap_request.proto",
        "basket_service_requests/ru_request.proto",
        "basket_balancer_requests/cb_request.proto",
        "basket_service.proto",
        "basket_balancer.proto",
        "external.proto",
    ];

    let include = ["", "basket_service_requests", "basket_balancer_requests"]
        .map(|include_dir| format!("{}/{}", proto_dir_path, include_dir));

    let proto_paths = proto_files.map(|proto_file| format!("{}/{}", proto_dir_path, proto_file));

    for proto_path in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto_path);
    }

    tonic_build::configure()
        .compile_protos(&proto_paths, &include)
        .expect("Failed to compile proto files");
}
