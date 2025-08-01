use hyperliquid_rust_sdk::{BaseUrl, InfoClient, Message, Subscription};
use log::info;
use tokio::{
    spawn,
    sync::mpsc::unbounded_channel,
    time::{sleep, Duration},
};

pub struct TestWsIsDelay {
    pub asset: String,
    pub latest_mid_price: f64,
    pub latest_l2_mid_price: f64,
    pub time: u64,
    pub l2_lag: u64,
}

impl TestWsIsDelay {
    pub fn new(asset: String) -> TestWsIsDelay {
        TestWsIsDelay {
            asset,
            latest_mid_price: -1.0,
            latest_l2_mid_price: -1.0,
            time: 0,
            l2_lag: 0,
        }
    }

    pub async fn start(&mut self) {
        let (send, mut recv) = unbounded_channel();
        let mut info_client = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();
        let sub_all_mids = info_client
            .subscribe(Subscription::AllMids, send.clone())
            .await
            .unwrap();
        let sub_l2_books = info_client
            .subscribe(
                Subscription::L2Book {
                    coin: self.asset.clone(),
                },
                send,
            )
            .await
            .unwrap();

        spawn(async move {
            sleep(Duration::from_secs(60)).await;
            info!("Unsubscribing from all data");
            info_client.unsubscribe(sub_all_mids).await.unwrap();
            info_client.unsubscribe(sub_l2_books).await.unwrap();
        });

        loop {
            let message = recv.recv().await.unwrap();
            match message {
                Message::AllMids(all_mids) => {
                    self.latest_mid_price = all_mids
                        .data
                        .mids
                        .get(&self.asset)
                        .unwrap()
                        .parse::<f64>()
                        .unwrap();
                }
                Message::L2Book(l2_book) => {
                    let bid = l2_book.data.levels.first().unwrap(); // 0
                    let ask = l2_book.data.levels.last().unwrap(); // 1

                    let best_bid = bid.first().unwrap().px.parse::<f64>().unwrap();
                    let best_ask = ask.last().unwrap().px.parse::<f64>().unwrap();

                    self.latest_l2_mid_price = (best_bid + best_ask) / 2.0;

                    // Log time difference from now
                    let now = chrono::Utc::now().timestamp_millis() as u64;

                    self.time = l2_book.data.time;
                    self.l2_lag = now - l2_book.data.time;
                }
                _ => {
                    panic!("Unsupported message type");
                }
            }

            if self.latest_mid_price > 0.0 && self.latest_l2_mid_price > 0.0 {
                println!(
                    "MP1(ALL_MIDS): {:.4} | MP2(L2_BOOK): {:.4} | Time: {} | L2 Lag: {}",
                    self.latest_mid_price, self.latest_l2_mid_price, self.time, self.l2_lag
                )
            }
        }
    }
}

/// How to Run
///
/// ```bash
/// RUST_LOG=info cargo test test_ws_is_delay -- --nocapture
/// ```

#[tokio::test]
async fn test_ws_is_delay() {
    // Test to figure out that if allMids have delays

    env_logger::init();

    let mut test = TestWsIsDelay::new("HYPE".to_string());
    test.start().await;
}
