use std::collections::HashMap;
use twelf::reexports::serde_json;
use twelf::reexports::serde_json::json;

pub fn hash_and_sign_event() {
    compute_content_hash();
    strip_event();
    sign_json();
}

pub fn compute_content_hash() {
    canonicalize_json();
}

pub fn canonicalize_json() {
    let mut canon: HashMap<String, String> = HashMap::new();
    canon.insert("charlie".to_string(), "foo".to_string());
    canon.insert("alpha".to_string(), "bar".to_string());
    canon.insert("bravo".to_string(), "baz".to_string());

    let json = serde_json::to_string(&canon).unwrap();
    println!("{}", json);
    let values: serde_json::Value = serde_json::from_str(&json).unwrap();
    println!("{:?}", values);
    let json2 = values.to_string();
    println!("{}", json2);
    assert_eq!(json, json2);

    let mut values2 = json!({
        "charlie": "foo",
        "bravo": {
            "november": "baz",
            "delta": {
                "zulu": "smerd",
                "foxtrot": "phred"
            }
        },
        "alpha": "bar",
    });
    println!("{:?}", values2);
    values2.sort_all_objects();
    println!("{}", values2.to_string());
}

pub fn strip_event() {

}

pub fn sign_json() {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_json() {
        canonicalize_json();
    }
}
