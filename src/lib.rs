mod api;
mod error;
mod wrapper;

pub use error::*;
pub use wrapper::*;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::*;

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
