//! A TOML-parsing library
//!
//! This library is an implementation in Rust of a parser for TOML configuration
//! files [1]. It is focused around high quality errors including specific spans
//! and detailed error messages when things go wrong.
//!
//! This implementation currently passes the language agnostic [test suite][2].
//!
//! # Example
//!
//! ```
//! let toml = r#"
//!     [test]
//!     foo = "bar"
//! "#;
//!
//! let value = toml::Parser::new(toml).parse().unwrap();
//! println!("{:?}", value);
//! ```
//!
//! # Conversions
//!
//! This library also supports using the standard `Encodable` and `Decodable`
//! traits with TOML values. This library provides the following conversion
//! capabilities:
//!
//! * `String` => `toml::Value` - via `Parser`
//! * `toml::Value` => `String` - via `Display`
//! * `toml::Value` => rust object - via `Decoder`
//! * rust object => `toml::Value` - via `Encoder`
//!
//! Convenience functions for performing multiple conversions at a time are also
//! provided.
//!
//! [1]: https://github.com/mojombo/toml
//! [2]: https://github.com/BurntSushi/toml-test
//!
//! # Encoding support
//!
//! This crate optionally supports the [`rustc-serialize`] crate and also the
//! [`serde`] crate through respective feature names. The `rustc-serialize`
//! feature is enabled by default.
//!
//! [`rustc-serialize`]: http://github.com/rust-lang/rustc-serialize
//! [`serde`]: http://github.com/serde-rs/serde

#![doc(html_root_url = "https://docs.rs/toml/0.3")]
#![deny(missing_docs)]

extern crate serde;

mod value;
pub use value::{Value, Table, Array};

pub mod ser;
#[doc(no_inline)]
pub use ser::{to_string, to_vec, Serializer};
mod de;

// mod parser;
// #[cfg(any(feature = "rustc-serialize", feature = "serde"))]
// mod encoder;
// #[cfg(any(feature = "rustc-serialize", feature = "serde"))]
// mod decoder;
