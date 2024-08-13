use std::str::FromStr;

#[derive(Debug)]
pub struct Imei {
    pub digits: [u8; 15],
}

#[derive(Debug)]
pub struct Tac {
    pub digits: [u8; 8],
}

#[derive(Debug)]
pub struct PhoneInfo {
    pub imei: Imei,
    pub manufacturer: String,
    pub model: String,
}

impl From<crate::api::PhoneInfo> for PhoneInfo {
    fn from(info: crate::api::PhoneInfo) -> Self {
        Self {
            imei: Imei::from_str(&info.imei).unwrap(),
            manufacturer: info.brand_name,
            model: info.model,
        }
    }
}

#[derive(Debug)]
pub struct DigitParseError;

impl Imei {
    pub fn reporting_body(&self) -> &[u8; 2] {
        self.digits[0..2].try_into().unwrap()
    }

    pub fn model_identifier(&self) -> &[u8; 6] {
        self.digits[2..8].try_into().unwrap()
    }

    pub fn type_allocation_code(&self) -> &[u8; 8] {
        self.digits[0..8].try_into().unwrap()
    }

    pub fn serial_number(&self) -> &[u8; 6] {
        self.digits[8..14].try_into().unwrap()
    }

    pub fn check_digit(&self) -> u8 {
        self.digits[14]
    }

    fn is_valid(&self) -> bool {
        luhn_checksum(&self.digits) == self.check_digit()
    }
}

impl Tac {
    pub fn reporting_body(&self) -> &[u8; 2] {
        self.digits[0..=1].try_into().unwrap()
    }

    pub fn model_identifier(&self) -> &[u8; 6] {
        self.digits[2..=7].try_into().unwrap()
    }
}

impl From<Tac> for Imei {
    fn from(tac: Tac) -> Self {
        let mut imei_digits = [0u8; 15];
        imei_digits[..8].copy_from_slice(&tac.digits);
        imei_digits[14] = luhn_checksum(&tac.digits);

        Self {
            digits: imei_digits,
        }
    }
}

impl From<Imei> for Tac {
    fn from(imei: Imei) -> Self {
        Self {
            digits: *imei.type_allocation_code(),
        }
    }
}

impl FromStr for Imei {
    type Err = DigitParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match string_to_digits(s) {
            Some(digits) => Ok(Self { digits }),
            None => Err(DigitParseError),
        }
    }
}

impl FromStr for Tac {
    type Err = DigitParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match string_to_digits(s) {
            Some(digits) => Ok(Self { digits }),
            None => Err(DigitParseError),
        }
    }
}

impl ToString for Imei {
    fn to_string(&self) -> String {
        self.digits.iter().map(|d| d.to_string()).collect()
    }
}

impl ToString for Tac {
    fn to_string(&self) -> String {
        self.digits.iter().map(|d| d.to_string()).collect()
    }
}

fn string_to_digits<const N: usize>(s: &str) -> Option<[u8; N]> {
    let mut digits = [0u8; N];
    for (i, c) in s.chars().enumerate() {
        if let Some(digit) = c.to_digit(10) {
            digits[i] = digit as u8;
        } else {
            return None;
        }
    }

    Some(digits)
}

fn luhn_checksum(digits: &[u8]) -> u8 {
    let mut checksum = 0;
    for (i, digit) in digits.into_iter().enumerate() {
        let digit = *digit as u32;
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

    checksum as u8
}
