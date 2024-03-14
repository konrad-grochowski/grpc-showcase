//! e2e gRPC test.
//! Scenario
//! 1. store 10000 key-value pairs
//!     - validate status code is correct
//! 2. load all the keys stored in the previous step
//!     - validate the value for each one is correct
use futures::StreamExt;

use serde_json::json;

#[ignore = "Requires running docker compose bundle"]
#[tokio::test]
#[tracing_test::traced_test]
async fn e2e_store_load_test() -> anyhow::Result<()> {
    const FUTURES_NUM: usize = 10000;

    let ca = std::fs::read_to_string("../self-signed-certs/client/rootCA.crt")?;
    let ca = reqwest::Certificate::from_pem(&ca.into_bytes())?;

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .add_root_certificate(ca)
        .build()?;

    tracing::info!("Storing {FUTURES_NUM} keys...");
    storing_keys(FUTURES_NUM, client.clone()).await?;

    tracing::info!("Loading {FUTURES_NUM} keys...");
    loading_keys(FUTURES_NUM, client.clone()).await?;
    Ok(())
}

async fn storing_keys(num: usize, client: reqwest::Client) -> anyhow::Result<()> {
    futures::stream::iter(0..num)
        .for_each_concurrent(None, |x| {
            let client = client.clone();
            async move {
                if let Err(err) = tokio::task::spawn(single_store(
                    format!("key_{}", x),
                    format!("value_{}", x),
                    client,
                ))
                .await
                .expect("Tasks SHOULD NOT panic, only return errors")
                {
                    panic!("Task returned an error: {err}");
                }
            }
        })
        .await;
    Ok(())
}

async fn loading_keys(num: usize, client: reqwest::Client) -> anyhow::Result<()> {
    futures::stream::iter(0..num)
        .for_each_concurrent(None, |x| {
            let client = client.clone();
            async move {
                if let Err(err) = tokio::task::spawn(single_load(
                    format!("key_{}", x),
                    format!("value_{}", x),
                    client,
                ))
                .await
                .expect("Tasks SHOULD NOT panic, only return errors")
                {
                    panic!("Task returned an error: {err}");
                }
            }
        })
        .await;
    Ok(())
}

async fn single_store(key: String, value: String, client: reqwest::Client) -> anyhow::Result<()> {
    let store_request = json! {
        {
        "key": key,
        "value": value
        }
    };

    let _resp = client
        .post("https://localhost:3000/store")
        .json(&store_request)
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

async fn single_load(key: String, value: String, client: reqwest::Client) -> anyhow::Result<()> {
    let load_request = json! {
        {
        "key": key,
        }
    };

    let load_reply = json! {
        {
        "key": key,
        "value": value
        }
    };
    let resp: reqwest::Response = client
        .get("https://localhost:3000/load")
        .json(&load_request)
        .send()
        .await?
        .error_for_status()?;
    assert_eq!(resp.json::<serde_json::Value>().await?, load_reply);

    Ok(())
}
