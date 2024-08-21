use std::error::Error;
use std::fmt::Display;

use reqwest::{Error as ReqwestError, Response, StatusCode};

use crate::api::{
    ServiceCheckInvalidApiKeyResponseBody, ServiceCheckPendingResponseBody,
    ServiceCheckStandardResponseBody,
};

pub(crate) type Result<T> = std::result::Result<T, ServiceCheckError>;

// TODO: Maybe split these into enum with `Wrapper` and `Api` variants
#[derive(Debug)]
pub enum ServiceCheckError {
    RequestPending { history_id: String, ulid: String },
    InvalidImeiNumber,
    MissingApiKey,
    InvalidApiKey { detail: String },
    InvalidServiceID,
    UnknownRequestError { error: ReqwestError },
    UnknownApiError { error: Response },
}

impl PartialEq for ServiceCheckError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ServiceCheckError::RequestPending {
                    history_id: history_id_self,
                    ulid: ulid_self,
                },
                ServiceCheckError::RequestPending {
                    history_id: history_id_other,
                    ulid: ulid_other,
                },
            ) => history_id_self.eq(history_id_other) && ulid_self.eq(ulid_other),
            (ServiceCheckError::InvalidImeiNumber, ServiceCheckError::InvalidImeiNumber) => true,
            (ServiceCheckError::MissingApiKey, ServiceCheckError::MissingApiKey) => true,
            (
                ServiceCheckError::InvalidApiKey {
                    detail: detail_self,
                },
                ServiceCheckError::InvalidApiKey {
                    detail: detail_other,
                },
            ) => detail_self.eq(detail_other),
            (ServiceCheckError::InvalidServiceID, ServiceCheckError::InvalidServiceID) => true,
            (
                ServiceCheckError::UnknownRequestError { error: error_self },
                ServiceCheckError::UnknownRequestError { error: error_other },
            ) => format!("{:?}", error_self) == format!("{:?}", error_other),
            (
                ServiceCheckError::UnknownApiError { error: error_self },
                ServiceCheckError::UnknownApiError { error: error_other },
            ) => format!("{:?}", error_self) == format!("{:?}", error_other),
            _ => false,
        }
    }
}

impl Error for ServiceCheckError {}

impl Display for ServiceCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ServiceCheckError::RequestPending { .. } => {
                "request has not resolved yet and is pending"
            }
            ServiceCheckError::InvalidImeiNumber => {
                "IMEI or TAC number passed to wrapper is invalid"
            }
            ServiceCheckError::MissingApiKey => "API key was not provided",
            ServiceCheckError::InvalidApiKey { .. } => "API key is invalid",
            ServiceCheckError::InvalidServiceID => "service ID is invalid",
            ServiceCheckError::UnknownRequestError { .. } => "unknown error occurred with request",
            ServiceCheckError::UnknownApiError { .. } => {
                "unknown error occurred with API; wrapper may be out-of-date"
            }
        })
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
                Err(ServiceCheckError::RequestPending { history_id, ulid })
            }
            StatusCode::FORBIDDEN => Err(ServiceCheckError::MissingApiKey),
            StatusCode::UNAUTHORIZED => {
                let ServiceCheckInvalidApiKeyResponseBody { detail } = response
                    .json::<ServiceCheckInvalidApiKeyResponseBody>()
                    .await
                    .unwrap();
                Err(ServiceCheckError::InvalidApiKey { detail })
            }
            StatusCode::NOT_FOUND => Err(ServiceCheckError::InvalidServiceID),
            _ => Err(ServiceCheckError::UnknownApiError { error: response }),
        }
    }
}
