// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::str::FromStr;
use std::fmt;
use serde;
use util::{U256 as EthU256, U128 as EthU128, Uint};

macro_rules! impl_uint {
	($name: ident, $other: ident, $size: expr) => {
		/// Uint serialization.
		#[derive(Debug, Default, Clone, Copy, PartialEq, Hash)]
		pub struct $name($other);

		impl Eq for $name { }

		impl<T> From<T> for $name where $other: From<T> {
			fn from(o: T) -> Self {
				$name($other::from(o))
			}
		}

		impl FromStr for $name {
			type Err = <$other as FromStr>::Err;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				$other::from_str(s).map($name)
			}
		}

		impl Into<$other> for $name {
			fn into(self) -> $other {
				self.0
			}
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "{}", self.0)
			}
		}

		impl fmt::LowerHex for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "{:#x}", self.0)
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> where S: serde::Serializer {
				serializer.serialize_str(&format!("0x{}", self.0.to_hex()))
			}
		}

		impl serde::Deserialize for $name {
			fn deserialize<D>(deserializer: &mut D) -> Result<$name, D::Error>
			where D: serde::Deserializer {
				struct UintVisitor;

				impl serde::de::Visitor for UintVisitor {
					type Value = $name;

					fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: serde::Error {
						// 0x + len
						if value.len() > 2 + $size * 16 || value.len() < 2 {
							return Err(serde::Error::custom("Invalid length."));
						}

						if &value[0..2] != "0x" {
							return Err(serde::Error::custom("Use hex encoded numbers with 0x prefix."))
						}

						$other::from_str(&value[2..]).map($name).map_err(|_| serde::Error::custom("Invalid hex value."))
					}

					fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: serde::Error {
						self.visit_str(&value)
					}
				}

				deserializer.deserialize(UintVisitor)
			}
		}

	}
}

impl_uint!(U128, EthU128, 2);
impl_uint!(U256, EthU256, 4);


#[cfg(test)]
mod tests {
	use super::U256;
	use serde_json;

	type Res = Result<U256, serde_json::Error>;

	#[test]
	fn should_serialize_u256() {
		let serialized1 = serde_json::to_string(&U256(0.into())).unwrap();
		let serialized2 = serde_json::to_string(&U256(1.into())).unwrap();
		let serialized3 = serde_json::to_string(&U256(16.into())).unwrap();
		let serialized4 = serde_json::to_string(&U256(256.into())).unwrap();

		assert_eq!(serialized1, r#""0x0""#);
		assert_eq!(serialized2, r#""0x1""#);
		assert_eq!(serialized3, r#""0x10""#);
		assert_eq!(serialized4, r#""0x100""#);
	}

	#[test]
	fn should_fail_to_deserialize_decimals() {
		let deserialized1: Res = serde_json::from_str(r#""""#);
		let deserialized2: Res = serde_json::from_str(r#""0""#);
		let deserialized3: Res = serde_json::from_str(r#""10""#);
		let deserialized4: Res = serde_json::from_str(r#""1000000""#);
		let deserialized5: Res = serde_json::from_str(r#""1000000000000000000""#);

		assert!(deserialized1.is_err());
		assert!(deserialized2.is_err());
		assert!(deserialized3.is_err());
		assert!(deserialized4.is_err());
		assert!(deserialized5.is_err());
	}

	#[test]
	fn should_deserialize_u256() {
		let deserialized1: U256 = serde_json::from_str(r#""0x""#).unwrap();
		let deserialized2: U256 = serde_json::from_str(r#""0x0""#).unwrap();
		let deserialized3: U256 = serde_json::from_str(r#""0x1""#).unwrap();
		let deserialized4: U256 = serde_json::from_str(r#""0x01""#).unwrap();
		let deserialized5: U256 = serde_json::from_str(r#""0x100""#).unwrap();

		assert_eq!(deserialized1, U256(0.into()));
		assert_eq!(deserialized2, U256(0.into()));
		assert_eq!(deserialized3, U256(1.into()));
		assert_eq!(deserialized4, U256(1.into()));
		assert_eq!(deserialized5, U256(256.into()));
	}
}
