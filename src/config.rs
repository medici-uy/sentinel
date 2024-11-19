use std::sync::{LazyLock, OnceLock};

use serde::Deserialize;
use url::Url;

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load);

#[derive(Deserialize, Debug)]
pub struct Config {
    pub engine_url: Url,
    pub web_url: Url,
    pub engine_cluster_arn: String,
    pub engine_service_arn: String,
    pub status_table_name: String,
    pub from_email_address: String,
    pub to_email_address: String,
}

impl Config {
    pub fn load() -> Self {
        envy::from_env().expect("couldn't load config from env")
    }
}

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub static AWS_SDK_CONFIG: OnceLock<aws_config::SdkConfig> = OnceLock::new();
pub static AWS_SES_CLIENT: OnceLock<aws_sdk_sesv2::Client> = OnceLock::new();

pub static AWS_ECS_CLIENT: LazyLock<aws_sdk_ecs::Client> =
    LazyLock::new(|| aws_sdk_ecs::Client::new(aws_sdk_config()));
pub static AWS_DYNAMODB_CLIENT: LazyLock<aws_sdk_dynamodb::Client> =
    LazyLock::new(|| aws_sdk_dynamodb::Client::new(aws_sdk_config()));

pub async fn init() {
    let sdk_config = aws_config::load_from_env().await;

    AWS_SDK_CONFIG.get_or_init(|| sdk_config);
    AWS_SES_CLIENT.get_or_init(|| aws_sdk_sesv2::Client::new(aws_sdk_config()));
}

pub fn aws_sdk_config() -> &'static aws_config::SdkConfig {
    AWS_SDK_CONFIG.get().expect("failed to get AWS config")
}

pub fn aws_ses_client() -> &'static aws_sdk_sesv2::Client {
    AWS_SES_CLIENT.get().expect("failed to get AWS SES client")
}
