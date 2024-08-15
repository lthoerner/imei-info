use reqwest::Client;

use crate::api::ServiceCheckStandardResponseBody;
use crate::error::{Result, ServiceCheckError};
use crate::wrapper::Imei;

pub(crate) async fn check_imei_with_service(
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
