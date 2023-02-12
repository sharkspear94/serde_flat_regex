use serde::Deserialize;
use serde_flat_regex::{self, flat_regex};

#[flat_regex]
#[derive(Debug, Deserialize, PartialEq, Clone)]
struct Foo {
    id: u32,
    #[flat_regex(regex = r"[a-Z]")]
    rest: std::collections::HashMap<std::string::String, String>,
}

fn main() {}
