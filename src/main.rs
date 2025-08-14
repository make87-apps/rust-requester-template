use make87::encodings::{Encoder, ProtobufEncoder};
use make87::interfaces::zenoh::ZenohInterface;
use make87_messages::core::Header;
use make87_messages::google::protobuf::Timestamp;
use make87_messages::text::PlainText;
use std::error::Error;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();

    let message_encoder = ProtobufEncoder::<PlainText>::new();
    let zenoh_interface = ZenohInterface::from_default_env("zenoh")?;
    let session = zenoh_interface.get_session().await?;

    let requester = zenoh_interface
        .get_querier(&session, "message_endpoint")
        .await?;

    let mut ticker = interval(Duration::from_millis(1000));

    loop {
        ticker.tick().await;

        let message = PlainText {
            header: Some(Header {
                timestamp: Timestamp::get_current_time().into(),
                reference_id: 0,
                entity_path: "/".to_string(),
            }),
            body: "Hello, World! 🦀".to_string(),
        };

        let message_encoded = message_encoder.encode(&message)?;
        let replies = requester.get().payload(&message_encoded).await?;

        while let Ok(reply) = replies.recv_async().await {
            match reply.result() {
                Ok(sample) => {
                    let message_decoded = message_encoder.decode(&sample.payload().to_bytes());
                    match message_decoded {
                        Ok(msg) => log::info!("Received response: {:?}", msg),
                        Err(e) => log::error!("Decode error: {e}"),
                    }
                }
                Err(err) => {
                    let payload = err
                        .payload()
                        .try_to_string()
                        .unwrap_or_else(|e| e.to_string().into());
                    log::error!("Received error: {}", payload);
                }
            }
        }
    }
}
