use serde::Deserialize;
use serde_flat_regex::flat_regex;
use std::ffi::CString;

#[flat_regex]
#[derive(Debug, Deserialize, PartialEq, Clone)]
struct Foo<'a> {
    id: u32,
    #[flat_regex(
        regex = r"lanport(status|speed)_\d+",
        key_access = "some_modul::as_str"
    )]
    #[serde(borrow)]
    rest: std::collections::HashMap<CString, &'a str>,
}

pub mod some_modul {
    use std::str::Utf8Error;

    pub fn as_str(s: &String) -> Result<&str, Utf8Error> {
        Ok(s.as_str())
    }
}

fn main() {}
