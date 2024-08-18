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

    #[test]
    fn try_good_imei_from_int() {
        assert_eq!(
            Imei::try_from(1234567890i32),
            Ok(Imei {
                digits: [0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
            }),
        );

        assert_eq!(
            Imei::try_from(1234567890u32),
            Ok(Imei {
                digits: [0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
            }),
        );

        assert_eq!(
            Imei::try_from(123456789012345i64),
            Ok(Imei {
                digits: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
            }),
        );

        assert_eq!(
            Imei::try_from(123456789012345u64),
            Ok(Imei {
                digits: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
            }),
        );

        assert_eq!(
            Imei::try_from(123456789012345i128),
            Ok(Imei {
                digits: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
            }),
        );

        assert_eq!(
            Imei::try_from(123456789012345u128),
            Ok(Imei {
                digits: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
            }),
        );

        assert_eq!(
            Imei::try_from(123456789012345isize),
            Ok(Imei {
                digits: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
            }),
        );

        assert_eq!(
            Imei::try_from(123456789012345usize),
            Ok(Imei {
                digits: [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5],
            }),
        )
    }

    #[test]
    fn try_good_tac_from_int() {
        assert_eq!(
            Tac::try_from(12345678i32),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        );

        assert_eq!(
            Tac::try_from(12345678u32),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        );

        assert_eq!(
            Tac::try_from(12345678i64),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        );

        assert_eq!(
            Tac::try_from(12345678u64),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        );

        assert_eq!(
            Tac::try_from(12345678i128),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        );

        assert_eq!(
            Tac::try_from(12345678u128),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        );

        assert_eq!(
            Tac::try_from(12345678isize),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        );

        assert_eq!(
            Tac::try_from(12345678usize),
            Ok(Tac {
                digits: [1, 2, 3, 4, 5, 6, 7, 8],
            }),
        )
    }

    #[test]
    fn try_bad_imei_from_int() {
        assert_eq!(
            Imei::try_from(-1i32),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(-1i64),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(1234567890123456i64),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(1234567890123456u64),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(-1i128),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(1234567890123456i128),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(1234567890123456u128),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(-1isize),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(1234567890123456isize),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Imei::try_from(1234567890123456usize),
            Err(ImeiWrapperError::ValueOutOfRange)
        )
    }

    #[test]
    fn try_bad_tac_from_int() {
        assert_eq!(Tac::try_from(-1i32), Err(ImeiWrapperError::ValueOutOfRange));

        assert_eq!(Tac::try_from(-1i64), Err(ImeiWrapperError::ValueOutOfRange));

        assert_eq!(
            Tac::try_from(123456789i64),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Tac::try_from(123456789u64),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Tac::try_from(-1i128),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Tac::try_from(123456789i128),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Tac::try_from(123456789u128),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Tac::try_from(-1isize),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Tac::try_from(123456789isize),
            Err(ImeiWrapperError::ValueOutOfRange)
        );

        assert_eq!(
            Tac::try_from(123456789usize),
            Err(ImeiWrapperError::ValueOutOfRange)
        )
    }
}
