#![allow(unused)]

mod api;
pub mod error;
pub mod wrapper;

use std::collections::VecDeque;
use std::str::FromStr;

use api::{ServiceCheckStandardResponseBody, BASIC_IMEI_CHECK_SID};
use error::{Result, ServiceCheckError};
use wrapper::{Imei, PhoneInfo, Tac};

/// Get the basic information about a device (make and model) using its IMEI.
/// The IMEI is required to be a string because if it was a numerical type, leading zeroes would be truncated.
///
/// This method will return an error in the following cases, roughly arranged in order of likelihood:
/// - The API key and/or the IMEI are invalid
/// - The IMEI.info API returns a "pending" (202) response, requiring a second request to retrieve the information
/// - The request could not be built or parsed due to a logic error within this crate or `reqwest`
/// - The IMEI.info API has been updated with a breaking change since the last crate release
pub async fn get_imei_info(api_key: &str, imei: &str) -> Result<PhoneInfo> {
    let Ok(imei) = Imei::from_str(imei) else {
        return Err(ServiceCheckError::InvalidImeiNumber);
    };

    let response = api::check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &imei).await?;
    Ok(response.result.into())
}

/// Get the basic information about a device (make and model) using its TAC.
/// The TAC is required to be a string because if it was a numerical type, leading zeroes would be truncated.
/// Though almost all TACs start with `35` for the reporting body associated with the IMEI, the first two digits
/// must be included because they are technically still part of the TAC.
/// The call to the IMEI.info API used here is the same as in [`get_imei_info`], it simply uses the TAC to generate
/// a generic IMEI to be checked.
///
/// This method will return an error in the following cases, roughly arranged in order of likelihood:
/// - The API key and/or the TAC are invalid
/// - The IMEI.info API returns a "pending" (202) response, requiring a second request to retrieve the information
/// - The request could not be built or parsed due to a logic error within this crate or `reqwest`
/// - The IMEI.info API has been updated with a breaking change since the last crate release
pub async fn get_tac_info(api_key: &str, tac: &str) -> Result<PhoneInfo> {
    let Ok(tac) = Tac::from_str(tac) else {
        return Err(ServiceCheckError::InvalidImeiNumber);
    };

    let response =
        api::check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &Imei::from(tac)).await?;
    Ok(response.result.into())
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
