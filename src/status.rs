use std::collections::HashMap;
use std::sync::LazyLock;

use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use lambda_runtime::Error;
use medici_shared::status::engine::EngineStatus;
use reqwest::StatusCode;
use url::Url;

use super::config::{CONFIG, HTTP_CLIENT};
use super::helpers::{was_engine_deployed_recently, was_healthy};

const STATUS_TABLE_PK_VALUE: &str = "medici";

pub static ENGINE_STATUS_URL: LazyLock<Url> =
    LazyLock::new(|| CONFIG.engine_url.join("status").unwrap());

#[derive(Clone, Debug)]
pub struct Status {
    pub engine_status: Option<EngineStatus>,
    pub engine_error: Option<String>,

    pub web_status_code: Option<StatusCode>,
    pub web_error: Option<String>,

    pub checked_at: DateTime<Utc>,
}

impl Status {
    pub fn healthy(&self) -> bool {
        let Some(engine_status) = &self.engine_status else {
            return false;
        };

        let Some(web_status_code) = &self.web_status_code else {
            return false;
        };

        self.engine_error.is_none()
            && self.web_error.is_none()
            && engine_status.healthy()
            && web_status_code.is_success()
    }

    pub async fn did_change(&self) -> Result<bool, Error> {
        let was_healthy = was_healthy().await?;

        Ok((!was_healthy && self.healthy())
            || (was_healthy && !self.healthy() && !was_engine_deployed_recently().await?))
    }

    pub async fn check() -> Self {
        let mut status = Self::default();

        match get_engine_status().await {
            Ok(engine_status) => {
                status.engine_status.replace(engine_status);
            }
            Err(error) => {
                status.engine_error.replace(error.to_string());
            }
        };

        match get_web_status_code().await {
            Ok(web_status_code) => {
                status.web_status_code.replace(web_status_code);
            }
            Err(error) => {
                status.web_error.replace(error.to_string());
            }
        }

        status
    }
}

impl Default for Status {
    fn default() -> Self {
        Self {
            engine_status: Default::default(),
            engine_error: Default::default(),

            web_status_code: Default::default(),
            web_error: Default::default(),

            checked_at: Utc::now(),
        }
    }
}

impl From<Status> for HashMap<String, AttributeValue> {
    fn from(status: Status) -> Self {
        Self::from([
            ("pk".into(), AttributeValue::S(STATUS_TABLE_PK_VALUE.into())),
            ("healthy".into(), AttributeValue::Bool(status.healthy())),
            (
                "changed_at".into(),
                AttributeValue::S(status.checked_at.to_rfc3339()),
            ),
        ])
    }
}

async fn get_engine_status() -> Result<EngineStatus, Error> {
    Ok(HTTP_CLIENT
        .get(ENGINE_STATUS_URL.clone())
        .send()
        .await?
        .json()
        .await?)
}

async fn get_web_status_code() -> Result<reqwest::StatusCode, Error> {
    let web_status = HTTP_CLIENT
        .get(CONFIG.web_url.clone())
        .send()
        .await?
        .status();

    Ok(web_status)
}
