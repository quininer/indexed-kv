use std::fmt;
use std::error::Error as StdError;
use wasm_bindgen::JsValue;


pub type JsResult<T> = std::result::Result<T, JsError>;

pub struct JsError(pub JsValue);

impl From<JsValue> for JsError {
    #[inline]
    fn from(val: JsValue) -> JsError {
        JsError(val)
    }
}

impl From<String> for JsError {
    #[inline]
    fn from(val: String) -> JsError {
        JsError(JsValue::from(val))
    }
}

impl From<&'static str> for JsError {
    #[inline]
    fn from(val: &'static str) -> JsError {
        JsError(JsValue::from(val))
    }
}

impl fmt::Display for JsError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Debug for JsError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl StdError for JsError {}
