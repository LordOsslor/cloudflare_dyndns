use core::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(try_from = "String")]
pub struct MaxLenString<const N: usize>(pub String);
impl<const N: usize> TryFrom<String> for MaxLenString<N> {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        if s.len() > N {
            Err(format!(
                "Provided string '{}' of length {} exceeds max length of {}",
                s,
                s.len(),
                N
            ))
        } else {
            Ok(Self(s))
        }
    }
}
impl<const N: usize> Display for MaxLenString<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[serde(try_from = "u16")]
pub struct MinMaxValueU16<const MIN: u16, const MAX: u16>(pub u16);
impl<const MIN: u16, const MAX: u16> TryFrom<u16> for MinMaxValueU16<MIN, MAX> {
    type Error = String;
    fn try_from(i: u16) -> Result<Self, Self::Error> {
        if i > MAX {
            Err(format!("Provided int {} exceeds maximum value: {}", i, MAX))
        } else if i < MIN {
            Err(format!(
                "Provided int {} is smaller than allowed: {}",
                i, MIN
            ))
        } else {
            Ok(Self(i))
        }
    }
}
#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[serde(try_from = "u32")]
pub struct MinMaxValueU32<const MIN: u32, const MAX: u32>(pub u32);
impl<const MIN: u32, const MAX: u32> TryFrom<u32> for MinMaxValueU32<MIN, MAX> {
    type Error = String;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
        if i > MAX {
            Err(format!("Provided int {} exceeds maximum value: {}", i, MAX))
        } else if i < MIN {
            Err(format!(
                "Provided int {} is smaller than allowed: {}",
                i, MIN
            ))
        } else {
            Ok(Self(i))
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy)]
#[serde(try_from = "u32")]
pub struct TTLU32(pub u32);
impl TryFrom<u32> for TTLU32 {
    type Error = String;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(Self(1)),
            v if v >= 60 && v <= 86400 => Ok(Self(v)),
            _ => Err(format!("Invalid TTL int: {v}"))?,
        }
    }
}
