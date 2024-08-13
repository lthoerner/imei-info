use chrono::{DateTime, Utc};
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

pub(crate) const BASIC_IMEI_CHECK_SID: u32 = 0;
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
pub(crate) enum ServiceCheckStatus {
    Done,
    #[serde(rename = "In_progress")]
    InProgress,
    Completed,
    Rejected,
}

#[derive(Deserialize, Debug)]
pub(crate) struct PhoneInfo {
    pub(crate) imei: String,
    pub(crate) brand_name: String,
    pub(crate) model: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ServiceCheckStandardResponseBody {
    pub(crate) id: u32,
    pub(crate) ulid: Option<String>,
    pub(crate) status: ServiceCheckStatus,
    pub(crate) service: String,
    pub(crate) service_id: u32,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) imei: Option<String>,
    pub(crate) imei2: Option<String>,
    pub(crate) sn: Option<String>,
    pub(crate) phone_number: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) token_key: String,
    pub(crate) token_request_price: String,
    pub(crate) result: PhoneInfo,
    pub(crate) requested_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ServiceCheckPendingResponseBody {
    pub(crate) message: String,
    pub(crate) history_id: String,
    pub(crate) ulid: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct ServiceCheckInvalidApiKeyResponseBody {
    pub(crate) detail: String,
}
