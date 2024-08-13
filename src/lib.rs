use std::collections::VecDeque;
use std::error::Error;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use reqwest::Error as ReqwestError;
use reqwest::Response;
use reqwest::{Client, StatusCode};
use serde::Deserialize;

const SAMSUNG_INFO_CHECK_SID: u32 = 4;
const SAMSUNG_KNOX_INFO_CHECK_SID: u32 = 76;
const GOOGLE_INFO_CHECK_SID: u32 = 54;
const LG_INFO_CHECK_SID: u32 = 66;
const SONY_INFO_CHECK_SID: u32 = 80;
const XIAOMI_INFO_CHECK_SID: u32 = 84;
const XIAOMI_MI_LOCK_INFO_CHECK_SID: u32 = 86;

const APPLE_FMI_STATUS_SID: u32 = 0;
const APPLE_WARRANTY_CHECK_SID: u32 = 12;
const APPLE_SIMLOCK_CHECK_SID: u32 = 104;
const APPLE_CARRIER_LOCK_FMI_STATUS_SID: u32 = 2;
const APPLE_SOLD_BY_WARRANTY_COVERAGE_SID: u32 = 11;

const BASIC_IMEI_CHECK_SID: u32 = 0;
const BLACKLIST_SIMPLE_CHECK_SID: u32 = 27;
const BLACKLIST_PREMIUM_CHECK_SID: u32 = 3;
const CARRIER_LOOKUP_SID: u32 = 48;
const ESIM_INFO_CHECK_SID: u32 = 52;
const MAC_ADDRESS_CHECK_SID: u32 = 106;
const VERIZON_USA_CHECK_SID: u32 = 32;
const TMOBILE_USA_CHECK_SID: u32 = 31;
const LOST_DEVICE_ADD_SID: u32 = 100;
const LOST_DEVICE_CHECK_SID: u32 = 101;

#[derive(Deserialize, Debug)]
struct ImeiCheckServiceStandardResponse {
    id: u32,
    ulid: Option<String>,
    status: ImeiCheckStatus,
    service: String,
    service_id: u32,
    created_at: DateTime<Utc>,
    imei: Option<String>,
    imei2: Option<String>,
    sn: Option<String>,
    phone_number: Option<String>,
    text: Option<String>,
    token_key: String,
    token_request_price: String,
    result: ImeiPhoneInfo,
    requested_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct ImeiCheckServicePendingResponse {
    message: String,
    history_id: String,
    ulid: String,
}

#[derive(Deserialize, Debug)]
struct ImeiCheckServiceInvalidTokenResponse {
    detail: String,
}

#[derive(Deserialize, Debug)]
enum ImeiCheckStatus {
    Done,
    #[serde(rename = "In_progress")]
    InProgress,
    Completed,
    Rejected,
}

#[derive(Deserialize, Debug)]
struct ImeiPhoneInfo {
    imei: String,
    brand_name: String,
    model: String,
}

// TODO: Maybe split these into enum with `Wrapper` and `API` variants
#[derive(Debug)]
pub enum ImeiCheckServiceError {
    RequestPending { history_id: String, ulid: String },
    InvalidImeiNumber,
    MissingAPIKey,
    InvalidAPIKey { detail: String },
    InvalidServiceID,
    UnknownRequestError { error: ReqwestError },
    UnknownAPIError { error: Response },
}

impl Display for ImeiCheckServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ImeiCheckServiceError::*;
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

#[derive(Debug)]
pub struct PhoneInfo {
    pub imei: u64,
    pub manufacturer: String,
    pub model: String,
}

impl From<ImeiPhoneInfo> for PhoneInfo {
    fn from(info: ImeiPhoneInfo) -> Self {
        Self {
            imei: info.imei.parse::<u64>().unwrap(),
            manufacturer: info.brand_name,
            model: info.model,
        }
    }
}

async fn classify_response(
    response: Response,
) -> Result<ImeiCheckServiceStandardResponse, ImeiCheckServiceError> {
    match response.status() {
        StatusCode::OK => Ok(response
            .json::<ImeiCheckServiceStandardResponse>()
            .await
            .unwrap()),
        StatusCode::ACCEPTED => {
            let ImeiCheckServicePendingResponse {
                history_id, ulid, ..
            } = response
                .json::<ImeiCheckServicePendingResponse>()
                .await
                .unwrap();
            Err(ImeiCheckServiceError::RequestPending { history_id, ulid })
        }
        StatusCode::FORBIDDEN => Err(ImeiCheckServiceError::MissingAPIKey),
        StatusCode::UNAUTHORIZED => {
            let ImeiCheckServiceInvalidTokenResponse { detail } = response
                .json::<ImeiCheckServiceInvalidTokenResponse>()
                .await
                .unwrap();
            Err(ImeiCheckServiceError::InvalidAPIKey { detail })
        }
        StatusCode::NOT_FOUND => Err(ImeiCheckServiceError::InvalidServiceID),
        _ => Err(ImeiCheckServiceError::UnknownAPIError { error: response }),
    }
}

impl From<ReqwestError> for ImeiCheckServiceError {
    fn from(error: ReqwestError) -> Self {
        ImeiCheckServiceError::UnknownRequestError { error }
    }
}

fn is_valid_imei(imei: &str) -> bool {
    let digits = imei.chars();
    if digits.clone().count() != 15 || imei.parse::<u64>().is_err() {
        return false;
    }

    let mut digits = digits.collect::<VecDeque<char>>();
    let stated_checksum = digits.pop_back().unwrap().to_digit(10).unwrap();
    let trimmed_imei = digits.into_iter().collect::<String>();
    let Some(calculated_checksum) = luhn_checksum(&trimmed_imei) else {
        return false;
    };

    stated_checksum == calculated_checksum
}

fn make_tac_valid(tac: &str) -> Option<String> {
    if tac.chars().count() != 8 {
        return None;
    }

    let checksum = luhn_checksum(tac)?;
    return Some(format!("{tac}000000{checksum}"));
}

fn luhn_checksum(value: &str) -> Option<u32> {
    let digits = value.chars();
    for d in digits.clone() {
        if !d.is_digit(10) {
            return None;
        }
    }

    let digits = digits
        .map(|c| c.to_digit(10).unwrap())
        .collect::<VecDeque<u32>>();
    let mut checksum = 0;
    for (i, digit) in digits.into_iter().enumerate() {
        if i % 2 != 0 {
            let double_digit = digit * 2;
            if double_digit < 10 {
                checksum += double_digit;
                continue;
            } else {
                checksum += 1;
                checksum += double_digit - 10;
            }
        } else {
            checksum += digit;
        }
    }

    checksum = 10 - (checksum % 10);
    if checksum == 10 {
        checksum = 0;
    }

    Some(checksum)
}

pub async fn get_imei_info(api_key: &str, imei: &str) -> Result<PhoneInfo, ImeiCheckServiceError> {
    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, imei).await?;
    Ok(response.result.into())
}

pub async fn get_tac_info(api_key: &str, tac: &str) -> Result<PhoneInfo, ImeiCheckServiceError> {
    let Some(tac_imei) = make_tac_valid(tac) else {
        return Err(ImeiCheckServiceError::InvalidImeiNumber);
    };

    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &tac_imei).await?;
    Ok(response.result.into())
}

async fn check_imei_with_service(
    service_id: u32,
    api_key: &str,
    imei: &str,
) -> Result<ImeiCheckServiceStandardResponse, ImeiCheckServiceError> {
    if !is_valid_imei(imei) {
        return Err(ImeiCheckServiceError::InvalidImeiNumber);
    }

    let client = Client::new();
    let response = client
        .get(format!("https://dash.imei.info/api/check/{service_id}"))
        .query(&[("API_KEY", api_key), ("imei", imei)])
        .send()
        .await?;

    Ok(classify_response(response).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_IMEIS_IPHONE_14PM: [&str; 10] = [
        "355086752340133",
        "355086758484539",
        "355086755725009",
        "355086750444747",
        "355086755760006",
        "355086754980852",
        "355086752456558",
        "355086753022573",
        "355086750080509",
        "355086756157624",
    ];

    #[tokio::test]
    async fn get_iphone_14pm_info() {
        dotenvy::dotenv().unwrap();
        let api_key = std::env::var("API_KEY").unwrap();
        let phone_info = get_imei_info(&api_key, "355086752340133").await.unwrap();
        println!("{:#?}", phone_info);
    }

    #[test]
    fn valid_imei_check() {
        for sample_imei in SAMPLE_IMEIS_IPHONE_14PM {
            assert!(is_valid_imei(sample_imei));
        }
    }

    #[tokio::test]
    async fn valid_tac_check() {
        dotenvy::dotenv().unwrap();
        let api_key = std::env::var("API_KEY").unwrap();
        let phone_info = get_tac_info(&api_key, "35508675").await.unwrap();
        println!("{:#?}", phone_info);
    }
}
