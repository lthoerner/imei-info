#![allow(unused)]

mod api;
mod error;
mod wrapper;

use std::{collections::VecDeque, str::FromStr};

use reqwest::Client;

use api::{ServiceCheckStandardResponseBody, BASIC_IMEI_CHECK_SID};
use error::{Result, ServiceCheckError};
use wrapper::{Imei, PhoneInfo, Tac};

pub async fn get_imei_info(api_key: &str, imei: &str) -> Result<PhoneInfo> {
    let Ok(imei) = Imei::from_str(imei) else {
        return Err(ServiceCheckError::InvalidImeiNumber);
    };

    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &imei).await?;
    Ok(response.result.into())
}

pub async fn get_tac_info(api_key: &str, tac: &str) -> Result<PhoneInfo> {
    let Ok(tac) = Tac::from_str(tac) else {
        return Err(ServiceCheckError::InvalidImeiNumber);
    };

    let response = check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &Imei::from(tac)).await?;
    Ok(response.result.into())
}

async fn check_imei_with_service(
    service_id: u32,
    api_key: &str,
    imei: &Imei,
) -> Result<ServiceCheckStandardResponseBody> {
    let client = Client::new();
    let response = client
        .get(format!("https://dash.imei.info/api/check/{service_id}"))
        .query(&[("API_KEY", api_key), ("imei", &imei.to_string())])
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
            assert!(Imei::from_str(sample_imei).is_ok());
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
