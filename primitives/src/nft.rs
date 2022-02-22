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

use codec::{Decode, Encode};
use enumflags2::BitFlags;
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::RuntimeDebug;
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
use sp_runtime::DispatchResult;

pub type NFTBalance = u128;
pub type CID = Vec<u8>;
pub type Attributes = BTreeMap<Vec<u8>, Vec<u8>>;

#[repr(u8)]
#[derive(Encode, Decode, Clone, Copy, BitFlags, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub enum ClassProperty {
	/// Is token transferable
	Transferable = 0b00000001,
	/// Is token burnable
	Burnable = 0b00000010,
	/// Is minting new tokens allowed
	Mintable = 0b00000100,
	/// Is class properties mutable
	ClassPropertiesMutable = 0b00001000,
}

#[derive(Clone, Copy, PartialEq, Default, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Properties(pub BitFlags<ClassProperty>);

impl Eq for Properties {}
impl Encode for Properties {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}
impl Decode for Properties {
	fn decode<I: codec::Input>(input: &mut I) -> sp_std::result::Result<Self, codec::Error> {
		let field = u8::decode(input)?;
		Ok(Self(
			<BitFlags<ClassProperty>>::from_bits(field as u8).map_err(|_| "invalid value")?,
		))
	}
}

impl TypeInfo for Properties {
	type Identity = Self;

	fn type_info() -> Type {
		Type::builder()
			.path(Path::new("BitFlags", module_path!()))
			.type_params(vec![TypeParameter::new("T", Some(meta_type::<ClassProperty>()))])
			.composite(Fields::unnamed().field(|f| f.ty::<u8>().type_name("ClassProperty")))
	}
}


/// Abstraction over a non-fungible token system.
pub trait ReserveNFT<AccountId> {
	/// The NFT class identifier.
	type ClassId: Default + Copy;

	/// The NFT token identifier.
	type TokenId: Default + Copy;

	fn reserve(owner: &AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult;

	fn unreserve(owner: &AccountId, token: (Self::ClassId, Self::TokenId)) -> DispatchResult;
}
