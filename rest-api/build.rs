fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute("StoreRequest", "#[derive(serde::Deserialize)]")
        .type_attribute("LoadRequest", "#[derive(serde::Deserialize)]")
        .type_attribute("LoadReply", "#[derive(serde::Serialize)]")
        .compile(
            &["../shared-resources/proto/key_value_store.proto"],
            &["../shared-resources/proto"],
        )?;
    // tonic_build::compile_protos("../shared-resources/proto/key_value_store.proto")?;
    Ok(())
}
