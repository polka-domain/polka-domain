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

use codec::FullCodec;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	DispatchResult,
};
use sp_std::fmt::Debug;

/// Abstraction over a non-fungible token system.
pub trait NFT<AccountId> {
	/// The NFT class identifier.
	type ClassId: Default + Copy;

	/// The NFT token identifier.
	type TokenId: Default + Copy;

	/// The balance of account.
	type Balance: AtLeast32BitUnsigned
		+ FullCodec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default;

	/// The number of NFTs assigned to `who`.
	fn balance(who: &AccountId) -> Self::Balance;

	/// The owner of the given token ID. Returns `None` if the token does not
	/// exist.
	fn owner(token: (Self::ClassId, Self::TokenId)) -> Option<AccountId>;

	/// Transfer the given token ID from one account to another.
	fn transfer(
		from: &AccountId,
		to: &AccountId,
		token: (Self::ClassId, Self::TokenId),
	) -> DispatchResult;

	fn reserve(owner: &AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult;

	fn unreserve(owner: &AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult;
}
