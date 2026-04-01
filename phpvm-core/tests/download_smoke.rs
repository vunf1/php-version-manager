//! Network smoke test (ignored by default). Run:
//! `cargo test -p phpvm-core --test download_smoke -- --ignored`
use phpvm_core::PhpVersion;
use phpvm_core::provider::Provider;

#[tokio::test]
#[ignore = "network: run with cargo test -p phpvm-core --test download_smoke -- --ignored"]
async fn official_windows_php_zip_is_reachable() {
    let url = Provider::build_official_windows_php_zip_url(&PhpVersion::new(8, 3, 16), true);
    let client = reqwest::Client::builder()
        .user_agent("phpvm-smoke-test")
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .expect("client");

    let head = client.head(&url).send().await.expect("head send");
    if head.status().is_success() {
        return;
    }

    let get = client
        .get(&url)
        .header("Range", "bytes=0-0")
        .send()
        .await
        .expect("get send");
    let status = get.status();
    assert!(
        status == reqwest::StatusCode::PARTIAL_CONTENT || status.is_success(),
        "unexpected status {} for {}",
        status,
        url
    );
}
