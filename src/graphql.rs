use std::ops::Deref;
use std::str::FromStr;

use async_graphql::connection::CursorType;
use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use serde::{Deserialize, Serialize};

/// `Pxid` is an unique 16 bytes prefixed identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Pxid(crate::Pxid);

impl Pxid {
    pub fn into_inner(self) -> crate::Pxid {
        self.0
    }
}

impl From<crate::Pxid> for Pxid {
    fn from(pxid: crate::Pxid) -> Self {
        Self(pxid)
    }
}

impl Deref for Pxid {
    type Target = crate::Pxid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[Scalar]
impl ScalarType for Pxid {
    fn parse(data: Value) -> InputValueResult<Self> {
        match data.clone() {
            Value::String(inner) => {
                if let Ok(pxid) = crate::Pxid::from_str(&inner) {
                    return Ok(Self(pxid));
                }

                Err(InputValueError::expected_type(data))
            }
            _ => Err(InputValueError::expected_type(data)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

impl ToString for Pxid {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for Pxid {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::Pxid::from_str(s).map(Self)
    }
}

impl CursorType for Pxid {
    type Error = crate::error::Error;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        Pxid::from_str(s)
    }

    fn encode_cursor(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use async_graphql::ScalarType;

    use super::{Pxid, Value};

    #[test]
    fn validates_string_is_actual_pxid_instance() {
        let pxid_str = String::from("acct_9m4e2mr0ui3e8a215n4g");
        let stri_value = Value::String(pxid_str);
        let pxid_scalar = Pxid::parse(stri_value).unwrap();
        let value = pxid_scalar.to_string();

        assert_eq!(value, "acct_9m4e2mr0ui3e8a215n4g");
    }

    #[test]
    fn invalidates_string_pxid_instance() {
        let pxid_str = String::from("9m4e2mr0ui3e8a");
        let stri_value = Value::String(pxid_str);
        let pxid_str_scalar = Pxid::parse(stri_value);

        assert!(pxid_str_scalar.is_err());
    }
}
