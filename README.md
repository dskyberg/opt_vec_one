# opt_vec_one

This is a demo crate to demonstrate a problem I am having with serde.

## vec_or_one
Within many open standards that leverage JSON, it is common practice to establish
constraints in which a schema value defined as a collection can be serialized as either a vector or a single instance.

For example, given the following structure:

```rust
use serde::{Deserialize, Serialize};
use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Inner {
    pub item: String
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Outer {
    #[serde(with = "vec_or_one")]
    items: Vec<Inner>
}
```

the following JSON objects may be treated as equivilant:

```json
{
    "items": [{"item":"value 1"}]
}
```

```json
{
    "items": {"item":"value 1"}
}
```

The mod `vec_or_one` demonstrates a working method for deserializing the above. It also supports serialization, although I am sure many will object to its utility.  For those that object, one can simply use
`#[serde(deserialize_with = "one_or_vec")]`, rather than
`#[serde(with = "one_or_vec")]`.

## option_vec_or_one

Herein lies my problem.  Serializing works as expected.

Given:

```rust
use serde::{Deserialize, Serialize};
use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Inner {
    pub item: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct OptOuter {
    count: usize,
    #[serde(with = "option_vec_or_one", skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Inner>>,
}
```

Serializing produces the correct output.
```rust
    fn serialize_none_test() {
        let json = r#"{"count":0}"#;

        let outer = OptOuter {
            count: 0,
            items: None,
        };

        let result = serde_json::to_string(&outer).expect("Oops!");
        assert_eq!(&json, &result);
    }
```

However deserializing:

```rust
    fn deserialize_none_test() {
        let test1 = r#"
        {
            "count": 0
        }"#;

        let outer = OptOuter {
            count: 0,
            items: None,
        };

        let result: OptOuter = serde_json::from_str(test1).expect("Oops!");
        assert_eq!(&outer, &result);
    }
```
Generates `Oops!: Error("missing field 'items'", line: 4, column: 9)`