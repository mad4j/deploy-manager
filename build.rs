fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("roe/proto/deploy_manager.proto")?;
    Ok(())
}
