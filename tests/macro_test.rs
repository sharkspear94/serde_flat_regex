use std::ffi::{CStr, CString};
use std::{collections::HashMap, marker::PhantomData, str::Utf8Error};

use bson::bson;
use serde::Deserialize;
use serde_flat_regex::{self, flat_regex};

#[test]
fn compile() {
    #[flat_regex]
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    struct Foo<'a, 'c> {
        id: u32,
        #[flat_regex(regex = r"1123", key_access = "as_str")]
        #[serde(borrow)]
        rest: std::collections::HashMap<String, &'a str>,
        asd: PhantomData<&'c str>,
    }

    fn as_str(s: &String) -> Result<&str, Utf8Error> {
        Ok(s)
    }
}

#[test]
fn cstr() {
    #[flat_regex]
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    struct Foo<'a> {
        id: u32,
        #[flat_regex(regex = r"lanport(status|speed)_\d+", key_access = "as_str")]
        #[serde(borrow)]
        rest: std::collections::HashMap<CString, &'a str>,
    }

    fn as_str(s: &CStr) -> Result<&str, Utf8Error> {
        s.to_str()
    }
    let raw = r#"
    {
        "id": 123,
        "lanportspeed_0": "100",
        "lanportstatus_0": "UP",
        "lanportspeed_1": "",
        "lanportstatus_10": "DOWN",
        "othterfield": "ASD",
        "lanport": "ADDDD"
    }"#;

    let res: Foo = serde_json::from_str(raw).expect("from str failed");

    assert_eq!(res.rest.len(), 4)
}

#[test]
fn json() {
    #[flat_regex]
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    struct Foo {
        id: u32,
        #[flat_regex(regex = r"lanport(status|speed)_\d+")]
        rest: std::collections::HashMap<std::string::String, String>,
        #[serde(flatten)]
        tasdasd: HashMap<String, String>,
    }

    let raw = r#"
    {
        "id": 123,
        "lanportspeed_0": "100",
        "lanportstatus_0": "UP",
        "lanportspeed_1": "",
        "lanportstatus_10": "DOWN",
        "othterfield": "ASD",
        "lanport": "ADDDD"
    }"#;

    let res: Foo = serde_json::from_str(raw).expect("from str failed");

    assert_eq!(res.tasdasd.len(), 6);
    assert_eq!(res.rest.len(), 4)
}

#[test]
fn json_should_fail() {
    #[flat_regex]
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    struct Foo {
        id: u32,
        #[flat_regex(regex = r"lanport(status|speed)_\d+")]
        rest: std::collections::HashMap<std::string::String, String>,
    }

    let raw = r#"
    {
        "id": 123,
        "lanportspeed_0": 100,
        "lanportstatus_0": "UP",
        "lanportspeed_1": 0,
        "lanportstatus_10": "DOWN",
        "othterfield": "ASD",
        "lanport": "ADDDD"
    }"#;

    let res: Result<Foo, _> = serde_json::from_str(raw);
    
    assert!(res.is_err())
}

#[test]
fn json_borrow_two_lt() {
    #[flat_regex]
    #[derive(Debug, Deserialize, PartialEq, Clone)]
    struct Foo<'a, 'b> {
        id: u32,
        lanportspeed_0: String,
        #[serde(borrow)]
        #[flat_regex(regex = r"lanport(status|speed)_\d+")]
        rest: HashMap<&'a str, &'b str>,
    }

    let raw = r#"
    {
        "id": 123,
        "lanportspeed_0": "100",
        "lanportstatus_0": "UP",
        "lanportspeed_1": "",
        "lanportstatus_10": "DOWN",
        "othterfield": "ASD",
        "lanport": "ADDDD"
    }"#;

    let res: Foo = serde_json::from_str(raw).expect("from str failed");

    assert_eq!(res.rest.len(), 3)
}

#[test]
fn json_test() {
    #[allow(dead_code)]
    #[flat_regex]
    #[derive(Debug, Deserialize)]
    struct RouterStatus {
        id: u32,
        wifi_status: bool,
        #[flat_regex(regex = r"lanportstatus_\d+")]
        lanports: std::collections::HashMap<String, String>,
    }

    let raw = r#"{
        "id": 1,
        "wifi_status": true,
        "lanportstatus_0": "UP",
        "lanportstatus_1": "UP",
        "lanportstatus_2": "DOWN",
        "lanportspeed": "100"
    }"#;

    let router_status: RouterStatus = serde_json::from_str(raw).unwrap();

    assert_eq!(router_status.lanports.len(), 3)
}

#[test]
fn bson_test() {
    #[allow(dead_code)]
    #[flat_regex]
    #[derive(Debug, Deserialize)]
    struct RouterStatus {
        id: u32,
        wifi_status: bool,
        #[flat_regex(regex = r"lanportstatus_\d+")]
        lanports: std::collections::HashMap<String, String>,
    }

    let bson = bson!({
        "id": 1,
        "wifi_status": true,
        "lanportstatus_0": "UP",
        "lanportstatus_1": "UP",
        "lanportstatus_2": "DOWN",
        "lanportspeed": "100"
    });

    let router_status: RouterStatus = bson::from_bson(bson).unwrap();

    assert_eq!(router_status.lanports.len(), 3)
}

#[test]
fn enum_test() {
    #[flat_regex]
    #[derive(Debug, Deserialize, PartialEq)]
    enum RouterStatus {
        A(bool),
        B(u32),
        #[serde(rename = "c")]
        C {
            id: i32,
            #[flat_regex(regex = r"lanportstatus_\d+")]
            lanports: HashMap<String, String>,
        },
        D,
    }

    let bson = bson!({"c":{
        "id": 1,
        "lanportstatus_0": "UP",
        "lanportstatus_1": "UP",
        "lanportspeed_0": "100"
    }});

    let router_status: RouterStatus = bson::from_bson(bson).unwrap();

    assert_eq!(
        router_status,
        RouterStatus::C {
            id: 1,
            lanports: HashMap::from([
                ("lanportstatus_0".to_string(), "UP".to_string()),
                ("lanportstatus_1".to_string(), "UP".to_string())
            ])
        }
    )
}

#[test]
fn should_fail_at_compliltime() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fail/regex_fail.rs")
}
