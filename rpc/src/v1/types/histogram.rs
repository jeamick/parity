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
// along with Parity. If not, see <http://www.gnu.org/licenses/>.

//! Gas prices histogram.

use v1::types::U256;
use util::stats;

/// Values of RPC settings.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Histogram {
	/// Gas prices for bucket edges.
	#[serde(rename="bucketBounds")]
	pub bucket_bounds: Vec<U256>,
	/// Transacion counts for each bucket.
	pub counts: Vec<u64>,
}

impl From<stats::Histogram> for Histogram {
	fn from(h: stats::Histogram) -> Self {
		Histogram {
			bucket_bounds: h.bucket_bounds.into_iter().map(Into::into).collect(),
			counts: h.counts
		}
	}
}
