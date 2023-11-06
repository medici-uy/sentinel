use aws_smithy_types_convert::date_time::DateTimeExt;
use chrono::{DateTime, Utc};
use lambda_runtime::Error;

use super::config::*;
use super::status::Status;

async fn engine_last_deployed_at() -> Result<DateTime<Utc>, Error> {
    let services_response = AWS_ECS_CLIENT
        .describe_services()
        .cluster(&CONFIG.engine_cluster_arn)
        .services(&CONFIG.engine_service_arn)
        .send()
        .await?;

    let service = services_response
        .services()
        .first()
        .expect("service not found");

    let last_deployed_at = service
        .deployments()
        .first()
        .expect("deployment not found")
        .created_at()
        .expect("no created_at in deployment")
        .to_chrono_utc()?;

    Ok(last_deployed_at.into())
}

pub async fn was_engine_deployed_recently() -> Result<bool, Error> {
    let now = Utc::now();
    let last_deployed_at = engine_last_deployed_at().await?;
    let difference = now - last_deployed_at;

    Ok(difference.num_minutes() < 15)
}

pub async fn was_healthy() -> Result<bool, Error> {
    let query_output = AWS_DYNAMODB_CLIENT
        .query()
        .table_name(&CONFIG.status_table_name)
        .key_condition_expression("pk = :pk")
        .expression_attribute_names("pk", "status")
        .limit(1)
        .scan_index_forward(false)
        .send()
        .await?;

    let Some(last_status) = query_output.items().first() else {
        return Ok(true);
    };

    let last_healthy = last_status
        .get("healthy")
        .expect("no healthy attribute in status")
        .as_bool()
        .expect("couldn't convert healthy attribute to boolean");

    Ok(*last_healthy)
}

pub async fn insert_status_change(status: Status) -> Result<(), Error> {
    AWS_DYNAMODB_CLIENT
        .put_item()
        .table_name(&CONFIG.status_table_name)
        .set_item(Some(status.into()))
        .send()
        .await?;

    Ok(())
}
