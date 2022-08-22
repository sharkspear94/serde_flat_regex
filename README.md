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
## Applicable Collections

The collection for flattening must be a [serde-map type](https://docs.rs/serde/latest/serde/de/trait.MapAccess.html) and implement `Extend<(K,V)> + Default`. The key can be anything that implements `AsRef<str>`