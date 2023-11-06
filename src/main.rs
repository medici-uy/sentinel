mod config;
mod email;
mod helpers;
mod status;

use lambda_runtime::{service_fn, Error, LambdaEvent};

use config::*;
use email::send_email;
use helpers::{insert_status_change, was_engine_deployed_recently, was_healthy};
use status::Status;

async fn handler(_event: LambdaEvent<()>) -> Result<(), Error> {
    let was_healthy = was_healthy().await?;
    let status = Status::check().await;

    if (!was_healthy && status.healthy())
        || (was_healthy && !status.healthy() && !was_engine_deployed_recently().await?)
    {
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
