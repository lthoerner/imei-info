use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ApiPhoneInfo;

/// An IMEI number, represented using an array of digits to prevent integer over/underflow or leading-zero truncation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Imei {
    pub digits: [u8; 15],
}

/// A TAC number, represented using an array of digits to prevent integer over/underflow or leading-zero truncation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tac {
    pub digits: [u8; 8],
}

/// The basic information about a phone: its IMEI, make, and model.
/// This is generally used in a context where the IMEI is already known, but it is included for flexibility's sake.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneInfo {
    pub imei: Imei,
    pub manufacturer: String,
    pub model: String,
}

impl From<ApiPhoneInfo> for PhoneInfo {
    fn from(info: ApiPhoneInfo) -> Self {
        Self {
            imei: Imei::from_str(&info.imei).unwrap(),
            manufacturer: info.brand_name,
            model: info.model,
        }
    }
}

#[derive(Debug)]
pub enum ImeiWrapperError {
    CannotParseDigits,
    ChecksumDoesNotMatch,
}

impl Error for ImeiWrapperError {}

impl Display for ImeiWrapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ImeiWrapperError::CannotParseDigits => {
                "one or more characters in the string is not numeric"
            }
            ImeiWrapperError::ChecksumDoesNotMatch => {
                "the IMEI check digit does not match its Luhn checksum"
            }
        })
    }
}

impl Imei {
    /// Retrieve the reporting body code (the first two digits of the IMEI).
    pub fn reporting_body(&self) -> &[u8; 2] {
        self.digits[0..2].try_into().unwrap()
    }

    /// Retrieve the bare model identifier, excluding the reporting body code (digits 3 through 8 of the IMEI).
    pub fn model_identifier(&self) -> &[u8; 6] {
        self.digits[2..8].try_into().unwrap()
    }

    /// Retrieve the type allocation code (TAC), which is the reporting body code and model identifier (digits 1 through 8 of the IMEI).
    pub fn type_allocation_code(&self) -> &[u8; 8] {
        self.digits[0..8].try_into().unwrap()
    }

    /// Retrieve the unit serial number (digits 9 through 14 of the IMEI).
    pub fn serial_number(&self) -> &[u8; 6] {
        self.digits[8..14].try_into().unwrap()
    }

    /// Retrieve the check digit, which is used for validation using Luhn's algorithm (digit 15 of the IMEI).
    pub fn check_digit(&self) -> u8 {
        self.digits[14]
    }

    /// Check if the IMEI is numerically valid. This does *not* mean that the IMEI is actually linked to a corresponding real-world device.
    fn is_valid(&self) -> bool {
        luhn_checksum(&self.digits) == self.check_digit()
    }
}

impl Tac {
    /// Retrieve the reporting body code (the first two digits of the TAC).
    pub fn reporting_body(&self) -> &[u8; 2] {
        self.digits[0..=1].try_into().unwrap()
    }

    /// Retrieve the bare model identifier, excluding the reporting body code (digits 3 through 8 of the TAC).
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
    type Err = ImeiWrapperError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(digits) = string_to_digits(s) else {
            return Err(ImeiWrapperError::CannotParseDigits);
        };

        let imei = Self { digits };
        if !imei.is_valid() {
            return Err(ImeiWrapperError::ChecksumDoesNotMatch);
        }

        Ok(imei)
    }
}

impl FromStr for Tac {
    type Err = ImeiWrapperError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match string_to_digits(s) {
            Some(digits) => Ok(Self { digits }),
            None => Err(ImeiWrapperError::CannotParseDigits),
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
