use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use log::info;
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

/// How to Run
///
/// ```bash
/// RUST_LOG=info cargo test test_ws_all_mids -- --nocapture
/// ```

#[tokio::test]
async fn test_ws_all_mids() {
    env_logger::init();

    let mut info_client = InfoClient::new(None, Some(BaseUrl::Testnet)).await.unwrap();

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(Subscription::AllMids, sender)
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        info!("Unsubscribing from mids data");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    // This loop ends when we unsubscribe
    while let Some(Message::AllMids(all_mids)) = receiver.recv().await {
        info!("Received mids data: {all_mids:?}");
    }
}
