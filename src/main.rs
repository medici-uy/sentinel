mod config;
mod email;
mod helpers;
mod status;

use lambda_runtime::{
    service_fn,
    tracing::{debug, info},
    Error, LambdaEvent,
};
use serde::Deserialize;

use config::CONFIG;
use email::send_email;
use helpers::insert_status_change;
use status::Status;

#[derive(Deserialize)]
struct Payload {}

async fn handler(_event: LambdaEvent<Payload>) -> Result<(), Error> {
    let status = Status::check().await;
    debug!("current status: {status:?}");

    if status.did_change().await? {
        info!("status changed");

        insert_status_change(status.clone()).await?;
        send_email(status).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    config::init().await;

    lambda_runtime::run(service_fn(handler)).await
}
