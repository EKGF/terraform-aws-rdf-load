/// See https://github.com/awslabs/aws-lambda-rust-runtime for more info on Rust runtime for AWS Lambda
use lambda_runtime::{service_fn, Error as LambdaError, LambdaEvent};

pub use request::Request;
use {
    ekg_aws_util::lambda::LambdaResponse, ekg_identifier::EkgIdentifierContexts, serde_json::Value,
};

mod request;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    ekg_util::tracing::aws_lfn_init();
    // Get the AWS config
    let aws_config = aws_config::load_from_env().await;
    // Create the NeptuneData client
    let aws_neptunedata_client = ekg_aws_util::neptune::get_neptunedata_client(&aws_config)?;

    // Call the actual handler of the request
    let func = service_fn(move |req| handle_lambda_event(req, aws_neptunedata_client.clone()));
    lambda_runtime::run(func).await?;
    Ok(())
}

/// The actual handler of the Lambda request.
async fn handle_lambda_event(
    event: LambdaEvent<Value>,
    aws_neptunedata_client: aws_sdk_neptunedata::Client,
) -> Result<LambdaResponse, LambdaError> {
    tracing::trace!("Event {:#?}\n\n", event.clone());

    let (payload, _ctx) = event.into_parts();

    handle_lambda_payload(payload, aws_neptunedata_client).await
}

async fn handle_lambda_payload(
    payload: Value,
    aws_neptunedata_client: aws_sdk_neptunedata::Client,
) -> Result<LambdaResponse, LambdaError> {
    tracing::trace!(
        "Payload {}",
        serde_json::to_string_pretty(&payload)?
    );

    let request = serde_json::from_value::<crate::Request>(payload).map_err(|e| {
        tracing::error!("Error parsing request: {}", e);
        e
    })?;

    match handle_lambda_request(&request, aws_neptunedata_client).await {
        Ok(mut response) => {
            response.clean();
            tracing::info!("Response: {:}", serde_json::to_string(&response)?);
            Ok(response)
        },
        Err(error) => {
            tracing::error!("Error handling request: {:?}", error);
            Err(error.into())
        },
    }
}

async fn handle_lambda_request(
    request: &crate::Request,
    aws_neptunedata_client: aws_sdk_neptunedata::Client,
) -> Result<LambdaResponse, LambdaError> {
    let _identifier_contexts = EkgIdentifierContexts::from_env()?;
    let load_request = &request.load_request;

    tracing::info!(
        "Load request for RDF file {:}",
        load_request.source
    );

    let result = aws_neptunedata_client
        .start_loader_job()
        .source(&load_request.source)
        .format(load_request.format.as_str().into())
        .iam_role_arn(&load_request.iam_role_arn)
        .mode(load_request.mode.clone().into())
        .s3_bucket_region(load_request.region.as_str().into())
        .fail_on_error(load_request.fail_on_error.as_str() == "TRUE")
        .parallelism(load_request.parallelism.as_str().into())
        .set_parser_configuration(Some(
            load_request.parser_configuration.as_hash_map(),
        ))
        .update_single_cardinality_properties(
            load_request.update_single_cardinality_properties.as_str() == "TRUE",
        )
        .queue_request(load_request.queue_request)
        .set_dependencies(Some(load_request.dependencies.clone()))
        .send()
        .await;

    match result {
        Ok(ref loader_job) => Ok(loader_job.into()),
        Err(error) => Ok(error.into()),
    }
}
