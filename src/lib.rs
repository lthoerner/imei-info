#![allow(unused)]

mod api;
mod error;

pub use WrapperPhoneInfo as PhoneInfo;

use std::collections::VecDeque;

use reqwest::Client;

use api::{PhoneInfo as ApiPhoneInfo, ServiceCheckStandardResponseBody, BASIC_IMEI_CHECK_SID};
use error::{Result, ServiceCheckError};

#[derive(Debug)]
pub struct WrapperPhoneInfo {
    pub imei: u64,
    pub manufacturer: String,
    pub model: String,
}

impl From<ApiPhoneInfo> for PhoneInfo {
    fn from(info: ApiPhoneInfo) -> Self {
        Self {
            imei: info.imei.parse::<u64>().unwrap(),
            manufacturer: info.brand_name,
            model: info.model,
        }
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

pub async fn get_imei_info(api_key: &str, imei: &str) -> Result<PhoneInfo> {
    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, imei).await?;
    Ok(response.result.into())
}

pub async fn get_tac_info(api_key: &str, tac: &str) -> Result<PhoneInfo> {
    let Some(tac_imei) = make_tac_valid(tac) else {
        return Err(ServiceCheckError::InvalidImeiNumber);
    };

    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &tac_imei).await?;
    Ok(response.result.into())
}

async fn check_imei_with_service(
    service_id: u32,
    api_key: &str,
    imei: &str,
) -> Result<ServiceCheckStandardResponseBody> {
    if !is_valid_imei(imei) {
        return Err(ServiceCheckError::InvalidImeiNumber);
    }

    let client = Client::new();
    let response = client
        .get(format!("https://dash.imei.info/api/check/{service_id}"))
        .query(&[("API_KEY", api_key), ("imei", imei)])
        .send()
        .await?;

    Ok(ServiceCheckError::classify_response(response).await?)
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
