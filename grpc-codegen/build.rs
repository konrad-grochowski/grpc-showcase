fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute("StoreRequest", "#[derive(serde::Deserialize)]")
        .type_attribute("LoadRequest", "#[derive(serde::Deserialize)]")
        .type_attribute("LoadReply", "#[derive(serde::Serialize)]")
        .compile(
            &["proto/key_value_storage.proto"],
            &["../shared-resources/proto"],
        )?;
    Ok(())
}
