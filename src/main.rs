mod config;
mod email;
mod helpers;
mod status;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::Deserialize;

use config::CONFIG;
use email::send_email;
use helpers::insert_status_change;
use status::Status;

#[derive(Deserialize)]
struct Payload {}

async fn handler(_event: LambdaEvent<Payload>) -> Result<(), Error> {
    println!("1");
    let status = Status::check().await;
    println!("2");

    if status.did_change().await? {
        println!("3");
        insert_status_change(status.clone()).await?;
        println!("4");
        send_email(status).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    config::init().await;

    lambda_runtime::run(service_fn(handler)).await
}
