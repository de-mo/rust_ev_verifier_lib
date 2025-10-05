// Copyright © 2025 Denis Morel
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License and
// a copy of the GNU General Public License along with this program. If not, see
// <https://www.gnu.org/licenses/>.

//! Module to define the methods used to generate deserialized functions
//!

use std::borrow::Cow;

use super::common_types::CiphertextDef;
use chrono::NaiveDateTime;
use rust_ev_system_library::rust_ev_crypto_primitives::prelude::{
    elgamal::Ciphertext, ByteArray, DecodeTrait, Hexa, Integer,
};
use serde::{
    de::{Deserialize as DeDeserialize, Deserializer, Error as SerdeError},
    Deserialize,
};

#[allow(dead_code)]
pub fn deserialize_string_hex_to_integer<'de, D>(deserializer: D) -> Result<Integer, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Integer;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: SerdeError,
        {
            Integer::from_hexa_string(v).map_err(|e| SerdeError::custom(e.to_string()))
        }
    }
    deserializer.deserialize_str(Visitor)
}

pub fn deserialize_string_base64_to_integer<'de, D>(deserializer: D) -> Result<Integer, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper<'a>(#[serde(borrow)] Cow<'a, str>);

    let buf: Wrapper = Deserialize::deserialize(deserializer)?;

    Integer::base64_decode(&buf.0).map_err(|e| SerdeError::custom(e.to_string()))
}

pub fn deserialize_option_string_base64_to_option_integer<'de, D>(
    deserializer: D,
) -> Result<Option<Integer>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper<'a>(#[serde(borrow)] Option<Cow<'a, str>>);

    let buf: Wrapper = Deserialize::deserialize(deserializer)?;

    match buf.0 {
        Some(buf) => ByteArray::base64_decode(&buf)
            .map_err(|e| SerdeError::custom(e.to_string()))
            .map(|e| Some(e.into_integer())),
        None => Ok(None),
    }
}

pub fn deserialize_string_to_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = NaiveDateTime;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: SerdeError,
        {
            NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S")
                .map_err(|e| SerdeError::custom(e.to_string()))
        }
    }
    deserializer.deserialize_str(Visitor)
}

#[allow(dead_code)]
pub fn deserialize_seq_string_hex_to_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Integer>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Integer>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element())? {
                let r_b = Integer::from_hexa_string(v).map_err(A::Error::custom)?;
                vec.push(r_b);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

#[allow(dead_code)]
pub fn deserialize_seq_string_base64_to_seq_bytearray<'de, D>(
    deserializer: D,
) -> Result<Vec<ByteArray>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<ByteArray>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            #[derive(Deserialize)]
            struct Wrapper<'a>(#[serde(borrow)] Cow<'a, str>);

            let mut vec = <Self::Value>::new();

            while let Some(wrapper) = seq.next_element::<Wrapper>()? {
                let r_b = ByteArray::base64_decode(&wrapper.0).map_err(A::Error::custom)?;
                vec.push(r_b);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

pub fn deserialize_seq_string_base64_to_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Integer>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Integer>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            #[derive(Deserialize)]
            struct Wrapper<'a>(#[serde(borrow)] Cow<'a, str>);

            let mut vec = <Self::Value>::new();

            while let Some(wrapper) = seq.next_element::<Wrapper>()? {
                let r_b = ByteArray::base64_decode(&wrapper.0).map_err(A::Error::custom)?;
                vec.push(r_b.into_integer());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

pub fn deserialize_seq_ciphertext<'de, D>(deserializer: D) -> Result<Vec<Ciphertext>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(with = "CiphertextDef")] Ciphertext);

    let v = Vec::deserialize(deserializer)?;
    Ok(v.into_iter().map(|Wrapper(a)| a).collect())
}

#[allow(dead_code)]
fn deserialize_seq_seq_string_hex_to_seq_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<Integer>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Vec<Integer>>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            #[derive(Deserialize)]
            struct Wrapper<'a>(#[serde(borrow)] Vec<Cow<'a, str>>);

            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element::<Wrapper>())? {
                let mut inner_vec = Vec::new();
                for x in v.0.iter() {
                    let r_b = Integer::from_hexa_string(x).map_err(A::Error::custom)?;
                    inner_vec.push(r_b);
                }
                vec.push(inner_vec.to_owned());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}

pub fn deserialize_seq_seq_string_base64_to_seq_seq_integer<'de, D>(
    deserializer: D,
) -> Result<Vec<Vec<Integer>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> ::serde::de::Visitor<'de> for Visitor {
        type Value = Vec<Vec<Integer>>;

        fn expecting(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(f, "a sequence of string")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            #[derive(Deserialize)]
            struct Wrapper<'a>(#[serde(borrow)] Vec<Cow<'a, str>>);

            let mut vec = <Self::Value>::new();

            while let Some(v) = (seq.next_element::<Wrapper>())? {
                let mut inner_vec: Vec<Integer> = Vec::new();
                for x in v.0.iter() {
                    let r_b = ByteArray::base64_decode(x).map_err(A::Error::custom)?;
                    inner_vec.push(r_b.into_integer());
                }
                vec.push(inner_vec.to_owned());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_seq(Visitor)
}
