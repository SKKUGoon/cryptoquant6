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
/// RUST_LOG=info cargo test test_ws_l2_book -- --nocapture
/// ```

#[tokio::test]
async fn test_ws_l2_book() {
    env_logger::init();

    let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();

    let (sender, mut receiver) = unbounded_channel();
    let subscription_id = info_client
        .subscribe(
            Subscription::L2Book {
                coin: "ETH".to_string(),
            },
            sender,
        )
        .await
        .unwrap();

    spawn(async move {
        sleep(Duration::from_secs(30)).await;
        info!("Unsubscribing from l2 book data");
        info_client.unsubscribe(subscription_id).await.unwrap()
    });

    // This loop ends when we unsubscribe
    while let Some(Message::L2Book(l2_book)) = receiver.recv().await {
        info!("Received l2 book data: {l2_book:?}");
    }
}
