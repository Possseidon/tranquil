use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub fn custom_id_encode<T: ?Sized + Serialize>(value: &T) -> String {
    let bincode = bincode::serialize(value).expect("custom_id serialization should not fail");
    String::from_utf8(base91::slice_encode(&bincode)).expect("base91 encoding should not fail")
}

pub fn custom_id_decode<T: DeserializeOwned>(custom_id: &str) -> Result<T> {
    let bincode = base91::slice_decode(custom_id.as_bytes());
    Ok(bincode::deserialize(&bincode)?)
}
