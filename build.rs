fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("external/roe/proto/deploy_manager.proto")?;
    tonic_build::compile_protos("external/roe/proto/managed_application.proto")?;
    Ok(())
}
