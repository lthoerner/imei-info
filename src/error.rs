use std::error::Error;
use std::fmt::Display;

use reqwest::{Error as ReqwestError, Response, StatusCode};

use crate::{
    ServiceCheckInvalidApiKeyResponseBody, ServiceCheckPendingResponseBody,
    ServiceCheckStandardResponseBody,
};

pub type Result<T> = std::result::Result<T, ServiceCheckError>;

// TODO: Maybe split these into enum with `Wrapper` and `API` variants
#[derive(Debug)]
pub enum ServiceCheckError {
    RequestPending { history_id: String, ulid: String },
    InvalidImeiNumber,
    MissingAPIKey,
    InvalidAPIKey { detail: String },
    InvalidServiceID,
    UnknownRequestError { error: ReqwestError },
    UnknownAPIError { error: Response },
}

impl Error for ServiceCheckError {}

impl Display for ServiceCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ServiceCheckError::*;
        f.write_str(match self {
            InvalidImeiNumber => "IMEI or TAC number passed to wrapper is invalid",
            InvalidAPIKey => "API key is invalid",
            InvalidServiceID => "service id is invalid",
            InvalidAPIValue => "value passed to API is invalid",
            RequestPending => "request has not resolved yet and is pending",
            UnknownRequestError => "unknown error occurred with request",
            UnknownAPIError => "unknown error occurred with API; wrapper may be out-of-date",
        })?;

        Ok(())
    }
}

impl From<ReqwestError> for ServiceCheckError {
    fn from(error: ReqwestError) -> Self {
        ServiceCheckError::UnknownRequestError { error }
    }
}

impl ServiceCheckError {
    pub(crate) async fn classify_response(
        response: Response,
    ) -> Result<ServiceCheckStandardResponseBody> {
        use ServiceCheckError::*;
        match response.status() {
            StatusCode::OK => Ok(response
                .json::<ServiceCheckStandardResponseBody>()
                .await
                .unwrap()),
            StatusCode::ACCEPTED => {
                let ServiceCheckPendingResponseBody {
                    history_id, ulid, ..
                } = response
                    .json::<ServiceCheckPendingResponseBody>()
                    .await
                    .unwrap();
                Err(RequestPending { history_id, ulid })
            }
            StatusCode::FORBIDDEN => Err(MissingAPIKey),
            StatusCode::UNAUTHORIZED => {
                let ServiceCheckInvalidApiKeyResponseBody { detail } = response
                    .json::<ServiceCheckInvalidApiKeyResponseBody>()
                    .await
                    .unwrap();
                Err(InvalidAPIKey { detail })
            }
            StatusCode::NOT_FOUND => Err(InvalidServiceID),
            _ => Err(UnknownAPIError { error: response }),
        }
    }
}
