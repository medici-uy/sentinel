use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;
use lambda_runtime::Error;
use medici_rust_shared::monitoring::Monitoring;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use url::Url;

use super::config::{CONFIG, HTTP_CLIENT};

const STATUS_TABLE_PK_VALUE: &str = "medici";

pub static ENGINE_MONITORING_URL: Lazy<Url> = Lazy::new(|| {
    CONFIG
        .engine_url
        .join("monitoring")
        .expect("invalid monitoring url")
});

#[derive(Default, Clone, Debug)]
pub struct Status {
    pub engine_monitoring: Option<Monitoring>,
    pub engine_error: Option<String>,

    pub web_status_code: Option<StatusCode>,
    pub web_error: Option<String>,
}

impl Status {
    pub fn healthy(&self) -> bool {
        let Some(engine_monitoring) = &self.engine_monitoring else {
            return false;
        };

        let Some(web_status_code) = &self.web_status_code else {
            return false;
        };

        return self.engine_error.is_none()
            && self.web_error.is_none()
            && engine_monitoring.ok()
            && web_status_code.is_success();
    }

    pub async fn check() -> Self {
        let mut status = Self::default();

        match get_engine_monitoring().await {
            Ok(engine_monitoring) => {
                status.engine_monitoring.replace(engine_monitoring);
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

impl From<Status> for HashMap<String, AttributeValue> {
    fn from(value: Status) -> Self {
        let now = Utc::now();

        Self::from([
            ("pk".into(), AttributeValue::S(STATUS_TABLE_PK_VALUE.into())),
            ("healthy".into(), AttributeValue::Bool(value.healthy())),
            ("date_time".into(), AttributeValue::S(now.to_rfc3339())),
        ])
    }
}

async fn get_engine_monitoring() -> Result<Monitoring, Error> {
    let engine_monitoring = HTTP_CLIENT
        .get(ENGINE_MONITORING_URL.clone())
        .send()
        .await?
        .json::<Monitoring>()
        .await?;

    Ok(engine_monitoring)
}

async fn get_web_status_code() -> Result<reqwest::StatusCode, Error> {
    let web_status = HTTP_CLIENT
        .get(CONFIG.web_url.clone())
        .send()
        .await?
        .status();

    Ok(web_status)
}
