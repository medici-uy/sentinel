mod config;
mod email;
mod helpers;
mod status;

use lambda_runtime::{service_fn, Error, LambdaEvent};

use config::*;
use email::send_email;
use helpers::insert_status_change;
use status::Status;

async fn handler(_event: LambdaEvent<()>) -> Result<(), Error> {
    let status = Status::check().await;

    if status.did_change().await? {
        insert_status_change(status.clone()).await?;
        send_email(status).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_aws().await;

    lambda_runtime::run(service_fn(handler)).await
}
