#[cfg(feature = "type_generation")]
use crate::response;

#[cfg(feature = "type_generation")]
pub fn get_typescript_definitions() -> String {
    [response::get_typescript_definitions()].concat().join("\n")
}
