fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/basket_balancer_notifier.proto")?;
    tonic_build::compile_protos("../proto/basket_service_processor.proto")?;

    Ok(())
}