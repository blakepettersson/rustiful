#[cfg(feature = "uuid")]
extern crate uuid;

use std::fmt;

#[cfg(feature = "uuid")]
use self::uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiId(JsonApiIdType);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum JsonApiIdType {
    Str(String),
    Int32(i32),
    Int64(i64),
    UInt32(u32),
    UInt64(u64),
    #[cfg(feature = "uuid")]
    Uuid(Uuid),
}

impl From<String> for JsonApiId {
    fn from(value: String) -> Self {
        JsonApiId(JsonApiIdType::Str(value))
    }
}

impl From<JsonApiId> for String {
    fn from(value: JsonApiId) -> Self {
        if let JsonApiIdType::Str(str) = value.0 {
            str
        } else {
            panic!("Expected String!")
        }
    }
}

impl From<i32> for JsonApiId {
    fn from(value: i32) -> Self {
        JsonApiId(JsonApiIdType::Int32(value))
    }
}

impl From<JsonApiId> for i32 {
    fn from(value: JsonApiId) -> Self {
        if let JsonApiIdType::Int32(val) = value.0 {
            val
        } else {
            panic!("Expected int32!")
        }
    }
}

impl From<i64> for JsonApiId {
    fn from(value: i64) -> Self {
        JsonApiId(JsonApiIdType::Int64(value))
    }
}

impl From<JsonApiId> for i64 {
    fn from(value: JsonApiId) -> Self {
        if let JsonApiIdType::Int64(val) = value.0 {
            val
        } else {
            panic!("Expected int64!")
        }
    }
}


impl From<u32> for JsonApiId {
    fn from(value: u32) -> Self {
        JsonApiId(JsonApiIdType::UInt32(value))
    }
}

impl From<JsonApiId> for u32 {
    fn from(value: JsonApiId) -> Self {
        if let JsonApiIdType::UInt32(val) = value.0 {
            val
        } else {
            panic!("Expected uint32!")
        }
    }
}

impl From<u64> for JsonApiId {
    fn from(value: u64) -> Self {
        JsonApiId(JsonApiIdType::UInt64(value))
    }
}

impl From<JsonApiId> for u64 {
    fn from(value: JsonApiId) -> Self {
        if let JsonApiIdType::UInt64(val) = value.0 {
            val
        } else {
            panic!("Expected uint64!")
        }
    }
}

#[cfg(feature = "uuid")]
impl From<Uuid> for JsonApiId {
    fn from(value: Uuid) -> Self {
        JsonApiId(JsonApiIdType::Uuid(value))
    }
}

#[cfg(feature = "uuid")]
impl From<JsonApiId> for Uuid {
    fn from(value: JsonApiId) -> Self {
        if let JsonApiIdType::Uuid(val) = value.0 {
            val
        } else {
            panic!("Expected uuid!")
        }
    }
}

impl fmt::Display for JsonApiId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::JsonApiIdType::*;

        match self.0 {
            Str(ref val) => write!(f, "{}", val),
            Int32(ref val) => write!(f, "{}", val),
            Int64(ref val) => write!(f, "{}", val),
            UInt32(ref val) => write!(f, "{}", val),
            UInt64(ref val) => write!(f, "{}", val),
            #[cfg(feature = "uuid")]
            Uuid(ref val) => write!(f, "{}", val),
        }
    }
}
