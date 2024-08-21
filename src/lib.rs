mod api;
mod error;
mod wrapper;

pub use error::*;
pub use wrapper::*;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::*;

    const SAMPLE_IMEIS_IPHONE_X: [&str; 10] = [
        "356741089728686",
        "356741088901532",
        "356741086755328",
        "356741084004687",
        "356741081753138",
        "356741082891218",
        "356741080856916",
        "356741082001776",
        "356741089053275",
        "356741089625577",
    ];

    const SAMPLE_IMEIS_IPHONE_11: [&str; 10] = [
        "356656424381449",
        "356656422684059",
        "356656420949041",
        "356656420595471",
        "356656426436027",
        "356656426168323",
        "356656425227245",
        "356656427178131",
        "356656420946799",
        "356656426277561",
    ];

    const SAMPLE_IMEIS_IPHONE_12: [&str; 10] = [
        "356741089728686",
        "356741088901532",
        "356741086755328",
        "356741084004687",
        "356741081753138",
        "356741082891218",
        "356741080856916",
        "356741082001776",
        "356741089053275",
        "356741089625577",
    ];

    const SAMPLE_IMEIS_IPHONE_13: [&str; 10] = [
        "353031119769899",
        "353031114733023",
        "353031119889093",
        "353031119201240",
        "353031117066769",
        "353031114152638",
        "353031119177119",
        "353031112524432",
        "353031112510548",
        "353031117786374",
    ];

    const SAMPLE_IMEIS_IPHONE_14: [&str; 10] = [
        "356663518856450",
        "356663512611182",
        "356663512926382",
        "356663514502413",
        "356663514265169",
        "356663516845729",
        "356663519115005",
        "356663512314985",
        "356663514250344",
        "356663511213253",
    ];

    const SAMPLE_IMEIS_IPHONE_15: [&str; 10] = [
        "357292743045215",
        "357292742100441",
        "357292742720628",
        "357292745630642",
        "357292748317395",
        "357292743231849",
        "357292749889186",
        "357292746950403",
        "357292744783558",
        "357292746107053",
    ];

    const SAMPLE_IMEIS_SAMSUNG_S10: [&str; 10] = [
        "351725105350612",
        "351725107128370",
        "351725102267413",
        "351725101223904",
        "351725108018653",
        "351725102151096",
        "351725101792791",
        "351725106100842",
        "351725101698170",
        "351725100178711",
    ];

    const SAMPLE_IMEIS_SAMSUNG_S20: [&str; 10] = [
        "355623112952700",
        "355623114222326",
        "355623115133407",
        "355623112353826",
        "355623114613797",
        "355623116570979",
        "355623114819972",
        "355623113858104",
        "355623111265039",
        "355623114676307",
    ];

    const SAMPLE_IMEIS_SAMSUNG_S21: [&str; 10] = [
        "359043379809723",
        "359043370680651",
        "359043375805360",
        "359043373735551",
        "359043370097328",
        "359043377664872",
        "359043371381523",
        "359043371473957",
        "359043375057335",
        "359043372667672",
    ];

    const SAMPLE_IMEIS_SAMSUNG_S22: [&str; 10] = [
        "351561161836263",
        "351561165123825",
        "351561168188320",
        "351561168458202",
        "351561169492820",
        "351561166778957",
        "351561162831321",
        "351561167705132",
        "351561166478160",
        "351561163409762",
    ];

    #[tokio::test]
    #[ignore]
    async fn get_iphone_info() {
        dotenvy::dotenv().unwrap();
        let api_key = std::env::var("API_KEY").unwrap();

        assert_eq!(
            get_imei_info(&api_key, SAMPLE_IMEIS_IPHONE_X[0]).await,
            Ok(PhoneInfo {
                imei: SAMPLE_IMEIS_IPHONE_X[0].try_into().unwrap(),
                manufacturer: "APPLE".to_owned(),
                model: "iPhone X".to_owned(),
            })
        );
    }

    #[tokio::test]
    #[ignore]
    async fn valid_tac_check() {
        dotenvy::dotenv().unwrap();
        let api_key = std::env::var("API_KEY").unwrap();
        let tac = &SAMPLE_IMEIS_SAMSUNG_S10[0][0..8];

        assert_eq!(
            get_tac_info(&api_key, tac).await,
            Ok(PhoneInfo {
                imei: Tac::from_str(tac).unwrap().into(),
                manufacturer: "SAMSUNG".to_owned(),
                model: "Galaxy S10 Exynos".to_owned(),
            })
        );
    }

    #[test]
    fn try_good_imei_from_string() {
        macro_rules! try_imeis {
            ( $prefix:expr; $( $postfix:expr ),* ) => {
                $(
                    paste::paste! {
                        for sample_imei in [<SAMPLE_IMEIS_ $prefix _ $postfix>] {
                            println!("{}", sample_imei);
                            let imei = Imei::from_str(sample_imei);
                            println!("{:#?}", &imei);
                            assert!(imei.is_ok());
                        }
                    }
                )*
            };
        }

        try_imeis!(IPHONE; X, 11, 12, 13, 14, 15);
        try_imeis!(SAMSUNG; S10, S20, S21, S22);
    }

    #[test]
    fn try_good_imei_from_int() {
        macro_rules! try_good_imei {
            ( $value:literal; $( $numeric_type:ty ),*; $separated:expr ) => {
                $(
                    assert_eq!(Imei::try_from($value as $numeric_type), Ok(Imei { digits: $separated }),);
                )*
            };
        }

        try_good_imei!(1234567897; i32, u32; [0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 7]);
        try_good_imei!(
            123456789012347; i64, u64, i128, u128, isize, usize;
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 7]
        );
    }

    #[test]
    fn try_good_tac_from_int() {
        macro_rules! try_good_tac {
            ( $value:literal; $( $numeric_type:ty ),*; $separated:expr ) => {
                $(
                    assert_eq!(Tac::try_from($value as $numeric_type), Ok(Tac { digits: $separated }),);
                )*
            };
        }

        try_good_tac!(12345678; i32, u32, i64, u64, i128, u128, isize, usize; [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn try_bad_imei_from_int() {
        macro_rules! try_bad_imei {
            ( $value:literal; $( $numeric_type:ty ),* ) => {
                $(
                    assert_eq!(Imei::try_from($value as $numeric_type), Err(ImeiWrapperError::ValueOutOfRange),);
                )*
            };
        }

        try_bad_imei!(-1; i32, i64, i128, isize);
        try_bad_imei!(1234567890123456; i64, u64, i128, u128, isize, usize);
    }

    #[test]
    fn try_bad_tac_from_int() {
        macro_rules! try_bad_tac {
            ( $value:literal; $( $numeric_type:ty ),* ) => {
                $(
                    assert_eq!(Tac::try_from($value as $numeric_type), Err(ImeiWrapperError::ValueOutOfRange),);
                )*
            };
        }

        try_bad_tac!(-1; i32, i64, i128, isize);
        try_bad_tac!(123456789; i32, u32, i64, u64, i128, u128, isize, usize);
    }
}
