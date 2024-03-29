// This file is part of Polka Domain.

// Copyright (C) 2021 Polka Domain.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod currency {
	use primitives::Balance;

	/// The existential deposit.
	pub const EXISTENTIAL_DEPOSIT: Balance = 1 * CENTS;

	pub const UNITS: Balance = 1_000_000_000_000;
	pub const CENTS: Balance = UNITS / 100;
	pub const MILLICENTS: Balance = CENTS / 1_000;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 2_000 * CENTS + (bytes as Balance) * 100 * MILLICENTS
	}
}

/// Fee-related
pub mod fee {
	use frame_support::weights::constants::{ExtrinsicBaseWeight, WEIGHT_PER_SECOND};
	use primitives::{
		currency::{TokenInfo, NAME},
		Balance, CurrencyId,
	};
	// use runtime_common::{cent, KAR};
	// use smallvec::smallvec;
	use sp_runtime::Perbill;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	// TODO: make those const fn
	pub fn dollar(currency_id: CurrencyId) -> Balance {
		10u128.saturating_pow(currency_id.decimals().expect("Not support Erc20 decimals").into())
	}

	pub fn cent(currency_id: CurrencyId) -> Balance {
		dollar(currency_id) / 100
	}

	fn base_tx_in_name() -> Balance {
		cent(NAME) / 10
	}

	/// Handles converting a weight scalar to a fee value, based on the scale
	/// and granularity of the node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, system::MaximumBlockWeight]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some
	/// examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	// pub struct WeightToFee;
	// impl WeightToFeePolynomial for WeightToFee {
	// 	type Balance = Balance;
	// 	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
	// 		// in Polka Domain, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
	// 		let p = base_tx_in_name();
	// 		let q = Balance::from(ExtrinsicBaseWeight::get());
	// 		smallvec![WeightToFeeCoefficient {
	// 			degree: 1,
	// 			negative: false,
	// 			coeff_frac: Perbill::from_rational(p % q, q),
	// 			coeff_integer: p / q,
	// 		}]
	// 	}
	// }

	pub fn ksm_per_second() -> u128 {
		let base_weight = Balance::from(ExtrinsicBaseWeight::get());
		let base_tx_per_second = (WEIGHT_PER_SECOND as u128) / base_weight;
		let kar_per_second = base_tx_per_second * base_tx_in_name();
		kar_per_second / 100
	}
}
