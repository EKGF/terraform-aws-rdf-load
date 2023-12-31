use aws_sdk_neptunedata::operation::get_loader_job_status::GetLoaderJobStatusError;

use {
    crate::lambda::LambdaDetailError,
    aws_sdk_neptunedata::{
        error::SdkError,
        operation::start_loader_job::{StartLoaderJobError, StartLoaderJobOutput},
        types::error::BadRequestException,
    },
    aws_smithy_runtime_api::client::result::TimeoutError,
    serde::{Deserialize, Serialize},
};

/// Generic response type that suits most of our lambda functions
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LambdaResponse {
    pub status_code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail_error: Option<LambdaDetailError>,
    /// A generic slot that can be used to pass back a result identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_identifier: Option<String>,
}

impl LambdaResponse {
    pub fn clean(&mut self) {
        // Remove the detailed message if it is the same as the message
        if Some(self.message.as_str()) == self.detailed_message.as_deref() {
            self.detailed_message = None;
        }
    }

    pub fn ok(message: &str, detailed_message: Option<&str>) -> Self {
        tracing::info!(message);
        Self {
            status_code: 200,
            message: message.to_string(),
            detailed_message: detailed_message.map(|s| s.to_string()),
            ..Default::default()
        }
    }
}

impl From<&BadRequestException> for LambdaResponse {
    fn from(error: &BadRequestException) -> Self {
        Self {
            status_code: 400,
            message: error
                .message
                .clone()
                .unwrap_or("unknown message".to_string()),
            detailed_message: Some(error.detailed_message.clone()),
            detail_error: LambdaDetailError::from_bad_request_exception(error),
            ..Default::default()
        }
    }
}

impl From<&StartLoaderJobError> for LambdaResponse {
    fn from(error: &StartLoaderJobError) -> Self {
        tracing::error!("Unknown service error: {:?}", error);
        Self {
            status_code: 500,
            message: format!("Service Error: {:?}", error),
            ..Default::default()
        }
    }
}

impl From<TimeoutError> for LambdaResponse {
    fn from(error: TimeoutError) -> Self {
        tracing::error!("Timeout Error: {:?}", error);
        Self {
            status_code: 504,
            message: "Timeout".to_string(),
            ..Default::default()
        }
    }
}

impl<R> From<SdkError<StartLoaderJobError, R>> for LambdaResponse {
    fn from(error: SdkError<StartLoaderJobError, R>) -> Self {
        match error {
            SdkError::ServiceError(service_error) => match service_error.err() {
                StartLoaderJobError::BadRequestException(exc) => exc.into(),
                source @ _ => source.into(),
            },
            SdkError::TimeoutError(timeout_error) => timeout_error.into(),
            _ => {
                tracing::error!("Unknown error starting the RDF load: {:}", error);
                Self {
                    status_code: 500,
                    message: format!("{:}", error),
                    ..Default::default()
                }
            },
        }
    }
}

impl From<&StartLoaderJobOutput> for LambdaResponse {
    fn from(loader_job: &StartLoaderJobOutput) -> Self {
        let load_id = loader_job.payload.get("loadId").cloned();
        Self {
            status_code: 200,
            message: "Loader job started successfully".to_string(),
            result_identifier: load_id,
            ..Default::default()
        }
    }
}

impl<R> From<SdkError<GetLoaderJobStatusError, R>> for LambdaResponse {
    fn from(error: SdkError<GetLoaderJobStatusError, R>) -> Self {
        match error {
            SdkError::ServiceError(service_error) => service_error.err().into(),
            SdkError::TimeoutError(timeout_error) => timeout_error.into(),
            _ => {
                tracing::error!(
                    "Unknown error checking the status of a loader job: {:}",
                    error
                );
                Self {
                    status_code: 500,
                    message: format!("{:}", error),
                    ..Default::default()
                }
            },
        }
    }
}

impl From<&GetLoaderJobStatusError> for LambdaResponse {
    fn from(error: &GetLoaderJobStatusError) -> Self {
        match error {
            GetLoaderJobStatusError::BadRequestException(exc) => exc.into(),
            source @ _ => {
                tracing::error!(
                    "Unknown service error checking the status of a loader job: {:}",
                    source
                );
                Self {
                    status_code: 500,
                    message: format!("{:}", source),
                    ..Default::default()
                }
            },
        }
    }
}
