#[cfg(feature = "deserialize")]
use serde::Deserialize;

#[cfg(feature = "serialize")]
use serde::Serialize;

#[cfg(feature = "type_generation")]
use specta::{ts, Type};

#[cfg_attr(feature = "type_generation", derive(Type))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Response<T> {
    pub(crate) data: T,
}

#[cfg_attr(feature = "type_generation", derive(Type))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct ErrorResponse<T> {
    pub(crate) message: String,
    pub(crate) code: Option<u16>,
    pub(crate) data: Option<T>,
}

#[cfg_attr(feature = "type_generation", derive(Type))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(
    any(feature = "serialize", feature = "deserialize"),
    serde(tag = "type")
)]
pub enum ResponseType<T> {
    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "success")
    )]
    Success(Response<T>),
    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "fail")
    )]
    Fail(Response<T>),
    #[cfg_attr(
        any(feature = "serialize", feature = "deserialize"),
        serde(rename = "error")
    )]
    Error(ErrorResponse<T>),
}

impl<T> ResponseType<T> {
    pub fn new_success(data: T) -> Self {
        ResponseType::Success(Response { data })
    }

    pub fn new_fail(data: T) -> Self {
        ResponseType::Fail(Response { data })
    }

    pub fn new_error(message: String) -> Self {
        ResponseType::Error(ErrorResponse {
            message,
            code: None,
            data: None,
        })
    }

    pub fn new_error_with_code(message: String, code: u16) -> Self {
        ResponseType::Error(ErrorResponse {
            message,
            code: Some(code),
            data: None,
        })
    }

    pub fn new_error_with_data(message: String, data: T) -> Self {
        ResponseType::Error(ErrorResponse {
            message,
            code: None,
            data: Some(data),
        })
    }

    pub fn new_error_with_code_and_data(message: String, code: u16, data: T) -> Self {
        ResponseType::Error(ErrorResponse {
            message,
            code: Some(code),
            data: Some(data),
        })
    }
}

#[cfg(feature = "type_generation")]
pub(crate) fn get_typescript_definitions() -> Vec<String> {
    vec![
        ts::export::<Response<()>>(&Default::default()).unwrap_or_default(),
        ts::export::<ErrorResponse<()>>(&Default::default()).unwrap_or_default(),
        ts::export::<ResponseType<()>>(&Default::default()).unwrap_or_default(),
    ]
}
