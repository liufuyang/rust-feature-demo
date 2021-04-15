fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/trackinfo.proto")?;
    tonic_build::compile_protos("proto/spotify/metadata/v1beta1/service.proto")?;
    Ok(())
}
