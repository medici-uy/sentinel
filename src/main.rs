mod config;
mod helpers;

use lambda_runtime::{service_fn, Error, LambdaEvent};
use medici_rust_shared::monitoring::Monitoring;
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use url::Url;

use config::*;
use helpers::{was_engine_deployed_recently, was_healthy};

pub static ENGINE_MONITORING_URL: Lazy<Url> = Lazy::new(|| {
    CONFIG
        .engine_url
        .join("monitoring")
        .expect("invalid monitoring url")
});

pub static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);

#[derive(Default, Debug)]
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
}

async fn handler(_event: LambdaEvent<()>) -> Result<(), Error> {
    let was_healthy = was_healthy().await?;
    let status = check().await;

    if !was_healthy && status.healthy() {
        todo!();
    } else if was_healthy && !status.healthy() && !was_engine_deployed_recently().await? {
        todo!();
    }

    Ok(())
}

async fn check() -> Status {
    let mut status = Status::default();

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    init_aws().await;

    lambda_runtime::run(service_fn(handler)).await
}
