mod vec_or_one {

    use serde::{self, de, Deserialize, Serialize, Serializer};

    #[derive(Deserialize, Debug)]
    #[serde(untagged)] // This is the magic. see https://serde.rs/enum-representations.html
    pub enum VecOrOne<T> {
        Vec(Vec<T>),
        One(T),
    }

    pub fn deserialize<'de, D: de::Deserializer<'de>, T: Deserialize<'de>>(
        de: D,
    ) -> Result<Vec<T>, D::Error> {
        use de::Deserialize as _;
        match VecOrOne::deserialize(de)? {
            VecOrOne::Vec(v) => Ok(v),
            VecOrOne::One(i) => Ok(vec![i]),
        }
    }

    pub fn serialize<S: Serializer, T: Serialize>(v: &Vec<T>, s: S) -> Result<S::Ok, S::Error> {
        match v.len() {
            1 => T::serialize(v.first().unwrap(), s),
            _ => Vec::<T>::serialize(v, s),
        }
    }
}

mod opt_vec_or_one {
    use serde::{self, de, Deserialize, Serialize, Serializer};
    use std::fmt;
    use std::marker::PhantomData;

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: de::Deserializer<'de>,
    {
        struct OptionVec<T>(PhantomData<Vec<T>>);

        impl<'de, T> de::Visitor<'de> for OptionVec<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Option<Vec<T>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a null, an array or map")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                super::vec_or_one::deserialize(deserializer).map(Some)
            }
        }

        deserializer.deserialize_option(OptionVec(PhantomData))
    }

    pub fn serialize<S: Serializer, T: Serialize>(
        ov: &Option<Vec<T>>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        match ov {
            Some(v) => match v.len() {
                1 => T::serialize(v.first().unwrap(), s),
                _ => Vec::<T>::serialize(v, s),
            },
            None => Option::<Vec<T>>::serialize(ov, s),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Inner {
        pub item: String,
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Outer {
        #[serde(with = "super::vec_or_one")]
        pub inners: Vec<Inner>,
    }

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct OptOuter {
        count: usize,
        #[serde(with = "super::opt_vec_or_one")]
        pub inners: Option<Vec<Inner>>,
    }

    #[test]
    fn deserialize_none_test() {
        let test1 = r#"
        {
            "count": 0
        }"#;

        let outer = OptOuter {
            count: 0,
            inners: None,
        };

        let result: OptOuter = serde_json::from_str(test1).expect("Oops!");
        assert_eq!(&outer, &result);
    }

    #[test]
    fn deserialize_vec_or_one_single() {
        let test1 = r#"
        {
            "inners": {
                "item": "value"
            }
        }"#;

        let inners = vec![Inner {
            item: "value".to_string(),
        }];
        let outer = Outer { inners };

        let result: Outer = serde_json::from_str(test1).expect("Oops!");
        assert_eq!(&outer, &result);
    }

    #[test]
    fn deserialize_vec_or_one_multple() {
        let test1 = r#"
        {
            "inners": [
                {
                "item": "value"
                },
                {
                    "item": "value"
                }
            ]
        }"#;

        let inners = vec![
            Inner {
                item: "value".to_string(),
            },
            Inner {
                item: "value".to_string(),
            },
        ];
        let outer = Outer { inners };

        let result: Outer = serde_json::from_str(test1).expect("Oops!");
        assert_eq!(&outer, &result);
    }

    #[test]
    fn serialize_vec_or_one() {
        let json = r##"{"inners":{"item":"value 1"}}"##;
        let outer = Outer {
            inners: vec![Inner {
                item: "value 1".to_string(),
            }],
        };
        let result = serde_json::from_str::<Outer>(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), outer);
    }

    #[test]
    fn serialize_vec_or_one_multiple() {
        let json = r##"{"inners":[{"item":"value 1"},{"item":"value 2"},{"item":"value 3"}]}"##;
        let outer = Outer {
            inners: vec![
                Inner {
                    item: "value 1".to_string(),
                },
                Inner {
                    item: "value 2".to_string(),
                },
                Inner {
                    item: "value 3".to_string(),
                },
            ],
        };
        let result = serde_json::to_string(&outer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json);
    }
}
