use aws_sdk_sesv2::types::Destination;
use chrono::{DateTime, Utc};
use lambda_runtime::Error;
use medici_shared::traits::EmailTemplate;
use serde::Serialize;

use super::config::AWS_SES_CLIENT;
use super::Status;
use super::CONFIG;

#[derive(Serialize, Clone, Debug)]
pub struct StatusChangeEmail {
    pub healthy: bool,
    pub changed_at: DateTime<Utc>,
}

impl EmailTemplate for StatusChangeEmail {
    const TEMPLATE_NAME: &'static str = "status_change";
}

impl From<Status> for StatusChangeEmail {
    fn from(status: Status) -> Self {
        StatusChangeEmail {
            healthy: status.healthy(),
            changed_at: Utc::now(),
        }
    }
}

pub async fn send_email(status: Status) -> Result<(), Error> {
    let template = StatusChangeEmail::from(status);
    let destination = Destination::builder()
        .to_addresses(&CONFIG.to_email_address)
        .build();

    AWS_SES_CLIENT
        .send_email()
        .from_email_address(&CONFIG.from_email_address)
        .destination(destination)
        .content(template.email_content())
        .send()
        .await?;

    Ok(())
}
