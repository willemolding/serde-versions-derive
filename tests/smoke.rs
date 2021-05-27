use serde::{Deserialize, Serialize};
use struct_versions_derive::version;

#[version(1)]
#[derive(Clone, Serialize, Deserialize)]
struct S {
    i: i32,
}

#[test]
fn works() {
    let vs = S { i: 11 }.to_versioned();
    assert_eq!(vs.version, 1);
}
