use serde::{Deserialize, Serialize};
use serde_versions_derive::serde_with_version;

#[serde_with_version(1)]
#[derive(Clone, Serialize, Deserialize)]
struct S {
    i: i32,
}

#[test]
fn adds_version_field() {
    let vs = S { i: 11 }.to_versioned();
    assert_eq!(vs.version, 1);
}


