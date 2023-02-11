# serde_flat_regex

A small macro for flattening a map type with regex machting keys.

## Example

```rust
#[flat_regex]
#[derive(Debug, Deserialize)]
struct RouterStatus {
    id: u32,
    wifi_status: bool,
    #[flat_regex(regex = r"lanportstatus_\d+")]
    lanports: std::collections::HashMap<String, String>,
}


#[test]
fn json_test() {
    let raw = r#"{
        "id": 1,
        "wifi_status": true,
        "lanportstatus_0": "UP",
        "lanportstatus_1": "UP",
        "lanportstatus_2": "DOWN",
        "lanportspeed": "100"
    }"#;

    let router_status: RouterStatus = serde_json::from_str(raw).unwrap();

    assert_eq!(router_status.lanports.len(),3)
}
```

The key can be anything that implements `AsRef<str>` or alternitiv the field attribute `key_access` can be set with a function returning a `Result<&str,_>`.

The function has to have the following signature: `fn key_access_fn_name<T>(key: &T) -> Result<&str,Error>` where `Error` can be any Errortype.


```rust
use std::collections::BTreeMap;
use std::str::Utf8Error;
use std::ffi::CString;
use serde_flat_regex::flat_regex;
use serde::Deserialize;

#[flat_regex]
#[derive(Debug,Deserialize)]   
struct RouterStatus {
    online: bool,
    #[flat_regex(regex = r"lanportstatus_\d+",key_access = "c_string_as_str")]
    lanport_status: BTreeMap<CString,bool>,
}

fn c_string_as_str(key: &CString) -> Result<&str,Utf8Error> {
    key.to_str()
}

let json = serde_json::json!({
    "online": true,
    "lanportstatus_0": true,
    "lanportspeed_0": "100", // no maching keys will be ignored
    "lanportstatus_1": false,
    "lanport_speed_1": "0", // no maching keys will be ignored
    "wifistatus": true
});

let res: RouterStatus = serde_json::from_value(json).unwrap();
assert_eq!(res.lanport_status.len(),2)
```

## Applicable Collections

The collection for flattening must be a [serde-map type](https://docs.rs/serde/latest/serde/de/trait.MapAccess.html) and implement `Extend<(K,V)> + Default`.