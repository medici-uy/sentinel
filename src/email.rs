use lambda_runtime::Error;

use super::Status;
use super::AWS_SES_CLIENT;

pub async fn send_email(status: Status) -> Result<(), Error> {
    AWS_SES_CLIENT.send_email().send().await?;

    Ok(())
}
