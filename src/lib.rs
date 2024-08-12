use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use reqwest::Error as ReqwestError;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use thiserror::Error;

const MAX_15_DIGIT: u64 = 999999999999999;

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
struct ImeiCheckServiceResponse {
    id: u32,
    ulid: Option<String>,
    status: ImeiCheckStatus,
    service: String,
    service_id: u32,
    created_at: DateTime<Utc>,
    imei: Option<String>,
    imei2: Option<String>,
    // ? Maybe `Option<u32>`?
    sn: Option<String>,
    phone_number: Option<String>,
    text: Option<String>,
    token_key: String,
    token_request_price: String,
    result: ImeiPhoneInfo,
    requested_at: DateTime<Utc>,
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

#[derive(Error, Debug)]
pub enum ImeiInfoError {
    #[error("IMEI or TAC number passed to SDK is invalid.")]
    InvalidInputNumber,
    #[error("Value passed to API is invalid.")]
    InvalidAPIValue { detail: String },
    #[error("Request has not resolved yet and is pending.")]
    RequestPending { history_id: String },
    #[error("API key is malformed.")]
    InvalidAPIKey,
    #[error("Requested service ID is invalid.")]
    InvalidServiceID,
    #[error("Unknown error occurred when resolving request.")]
    UnknownRequestError { error: ReqwestError },
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

impl From<ReqwestError> for ImeiInfoError {
    fn from(reqwest_error: ReqwestError) -> Self {
        if let Some(status_code) = reqwest_error.status() {
            return match status_code {
                StatusCode::ACCEPTED => Self::RequestPending {
                    history_id: "PLACEHOLDER".to_owned(),
                },
                StatusCode::UNAUTHORIZED => Self::InvalidAPIValue {
                    detail: "PLACEHOLDER".to_owned(),
                },
                StatusCode::FORBIDDEN => Self::InvalidAPIKey,
                StatusCode::NOT_FOUND => Self::InvalidServiceID,
                _ => Self::UnknownRequestError {
                    error: reqwest_error.without_url(),
                },
            };
        }

        return Self::UnknownRequestError {
            error: reqwest_error.without_url(),
        };
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

pub async fn get_imei_info(api_key: &str, imei: &str) -> Result<PhoneInfo, ImeiInfoError> {
    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, imei).await?;
    Ok(response.result.into())
}

pub async fn get_tac_info(api_key: &str, tac: &str) -> Result<PhoneInfo, ImeiInfoError> {
    let Some(tac_imei) = make_tac_valid(tac) else {
        return Err(ImeiInfoError::InvalidInputNumber);
    };

    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &tac_imei).await?;
    Ok(response.result.into())
}

async fn check_imei_with_service(
    service_id: u32,
    api_key: &str,
    imei: &str,
) -> Result<ImeiCheckServiceResponse, ImeiInfoError> {
    if !is_valid_imei(imei) {
        return Err(ImeiInfoError::InvalidInputNumber);
    }

    let client = Client::new();
    let response = client
        .get(format!("https://dash.imei.info/api/check/{service_id}"))
        .query(&[("API_KEY", api_key), ("imei", imei)])
        .send()
        .await?
        .json::<ImeiCheckServiceResponse>()
        .await?;

    Ok(response)
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
