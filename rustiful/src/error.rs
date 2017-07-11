use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiErrorArray {
    pub errors: Vec<JsonApiError>
}

impl JsonApiErrorArray {
    pub fn new<T: Error>(error: &T, status: u16) -> JsonApiErrorArray {
        JsonApiErrorArray {
            errors: vec![JsonApiError::new(error, status)]
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonApiError {
    pub title: String,
    pub status: String,
    pub detail: String
}

impl JsonApiError {
    pub fn new<T: Error>(error: &T, status: u16) -> JsonApiError {
        JsonApiError {
            title: error.description().to_string(),
            status: status.to_string(),
            detail: format!("{}", error)
        }
    }
}
