use reqwest;
use serde_json::json;

#[tokio::test]
async fn key_storage() -> anyhow::Result<()> {
    let ca = std::fs::read_to_string("../self-signed-certs/client/rootCA.crt")?;
    let ca = reqwest::Certificate::from_pem(&ca.into_bytes())?;

    let client = reqwest::Client::builder()
    .use_rustls_tls()
    .add_root_certificate(ca).build()?;

    let resp = client.post("https://localhost:3000/store").json(&json!{
        {
        "key": "key",
        "value": "value"
        }
    }).send().await?;
    

    let resp: reqwest::Response = client.get("https://localhost:3000/load").json(&json!{
        {
        "key": "key",
        }
    }).send().await?;
    panic!("{:?}", resp.text().await?);

    Ok(())
}
