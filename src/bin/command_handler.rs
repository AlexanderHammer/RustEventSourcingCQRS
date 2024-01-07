use std::error::Error;
use eventstore::{Client, RetryOptions, SubscribeToAllOptions, SubscriptionFilter};

#[tokio::main]
async fn main(){
    let settings = "esdb://admin:changeit@localhost:2113?tls=false"
    .parse()
    .unwrap();
    let es_client = Client::new(settings).unwrap();
    read_all_events(es_client).await;
}   


async fn read_all_events(client: Client) -> Result<(), Box<dyn Error>>  {
    let retry = RetryOptions::default().retry_forever();
    let filter = SubscriptionFilter::on_event_type().exclude_system_events();
    let options = SubscribeToAllOptions::default().filter(filter).retry_options(retry);
    let mut sub = client.subscribe_to_all(&options).await;

    loop {
        let event = sub.next().await?;
        let stream_id = event.get_original_stream_id();
        let revision = event.get_original_event().revision;

        println!("Received event {}@{}", revision, stream_id);
    }
}