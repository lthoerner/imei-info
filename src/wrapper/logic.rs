use std::str::FromStr;

use crate::api::BASIC_IMEI_CHECK_SID;
use crate::error::{Result, ServiceCheckError};
use crate::wrapper::{Imei, PhoneInfo, Tac};

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

    let response =
        crate::api::check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &imei).await?;
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
        crate::api::check_imei_with_service(BASIC_IMEI_CHECK_SID, api_key, &Imei::from(tac))
            .await?;
    Ok(response.result.into())
}
