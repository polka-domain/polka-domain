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

#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks};
use frame_support::{dispatch::DispatchErrorWithPostInfo, traits::Get, weights::DispatchClass};
use frame_system::RawOrigin;
use primitives::Balance;
use sp_runtime::traits::{AccountIdConversion, StaticLookup, UniqueSaturatedInto};
use sp_std::vec;

pub use crate::*;

pub struct Module<T: Config>(crate::Pallet<T>);

const SEED: u32 = 0;

fn dollar(d: u32) -> Balance {
	let d: Balance = d.into();
	d.saturating_mul(1_000_000_000_000_000_000)
}

fn test_attr() -> Attributes {
	let mut attr: Attributes = BTreeMap::new();
	for i in 0..30 {
		attr.insert(vec![i], vec![0; 64]);
	}
	attr
}

fn create_token_class<T: Config>(
	caller: T::AccountId,
) -> Result<T::AccountId, DispatchErrorWithPostInfo> {
	let base_currency_amount = dollar(1000);
	<T as module::Config>::Currency::make_free_balance_be(
		&caller,
		base_currency_amount.unique_saturated_into(),
	);

	let module_account: T::AccountId =
		T::PalletId::get().into_sub_account(orml_nft::Pallet::<T>::next_class_id());
	crate::Pallet::<T>::create_class(
		RawOrigin::Signed(caller).into(),
		vec![1],
		Properties(
			ClassProperty::Transferable |
				ClassProperty::Burnable |
				ClassProperty::Mintable |
				ClassProperty::ClassPropertiesMutable,
		),
		test_attr(),
	)?;

	<T as module::Config>::Currency::make_free_balance_be(
		&module_account,
		base_currency_amount.unique_saturated_into(),
	);

	Ok(module_account)
}

benchmarks! {
	// create NFT class
	create_class {
		let caller: T::AccountId = account("caller", 0, SEED);
		let base_currency_amount = dollar(1000);

		<T as module::Config>::Currency::make_free_balance_be(&caller, base_currency_amount.unique_saturated_into());
	}: _(RawOrigin::Signed(caller), vec![1], Properties(ClassProperty::Transferable | ClassProperty::Burnable), test_attr())

	// mint NFT token
	mint {
		let i in 1 .. 1000;

		let caller: T::AccountId = account("caller", 0, SEED);
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to);

		let module_account = create_token_class::<T>(caller)?;
	}: _(RawOrigin::Signed(module_account), to_lookup, 0u32.into(), vec![1], test_attr(), i)

	// transfer NFT token to another account
	transfer {
		let caller: T::AccountId = account("caller", 0, SEED);
		let caller_lookup = T::Lookup::unlookup(caller.clone());
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to.clone());

		let module_account = create_token_class::<T>(caller)?;

		crate::Pallet::<T>::mint(RawOrigin::Signed(module_account).into(), to_lookup, 0u32.into(), vec![1], test_attr(), 1)?;
	}: _(RawOrigin::Signed(to), caller_lookup, (0u32.into(), 0u32.into()))

	// burn NFT token
	burn {
		let caller: T::AccountId = account("caller", 0, SEED);
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to.clone());

		let module_account = create_token_class::<T>(caller)?;

		crate::Pallet::<T>::mint(RawOrigin::Signed(module_account).into(), to_lookup, 0u32.into(), vec![1], test_attr(), 1)?;
	}: _(RawOrigin::Signed(to), (0u32.into(), 0u32.into()))

	// burn NFT token with remark
	burn_with_remark {
		let b in 0 .. *T::BlockLength::get().max.get(DispatchClass::Normal) as u32;
		let remark_message = vec![1; b as usize];
		let caller: T::AccountId = account("caller", 0, SEED);
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to.clone());

		let module_account = create_token_class::<T>(caller)?;

		crate::Pallet::<T>::mint(RawOrigin::Signed(module_account).into(), to_lookup, 0u32.into(), vec![1], test_attr(), 1)?;
	}: _(RawOrigin::Signed(to), (0u32.into(), 0u32.into()), remark_message)

	// destroy NFT class
	destroy_class {
		let caller: T::AccountId = account("caller", 0, SEED);
		let caller_lookup = T::Lookup::unlookup(caller.clone());

		let base_currency_amount = dollar(1000);

		let module_account = create_token_class::<T>(caller)?;

	}: _(RawOrigin::Signed(module_account), 0u32.into(), caller_lookup)

	update_class_properties {
		let caller: T::AccountId = account("caller", 0, SEED);
		let to: T::AccountId = account("to", 0, SEED);
		let to_lookup = T::Lookup::unlookup(to);

		let module_account = create_token_class::<T>(caller)?;
	}: _(RawOrigin::Signed(module_account), 0u32.into(), Properties(ClassProperty::Transferable.into()))
}

#[cfg(test)]
mod mock {
	use codec::{Decode, Encode};
	use frame_support::{
		parameter_types,
		traits::{Contains, InstanceFilter},
		weights::Weight,
		PalletId, RuntimeDebug,
	};
	use sp_core::{crypto::AccountId32, H256};
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	};

	use super::*;
	use crate as nft;

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::one();
	}

	pub type AccountId = AccountId32;

	impl frame_system::Config for Runtime {
		type AccountData = pallet_balances::AccountData<Balance>;
		type AccountId = AccountId;
		type BaseCallFilter = BaseFilter;
		type BlockHashCount = BlockHashCount;
		type BlockLength = ();
		type BlockNumber = u64;
		type BlockWeights = ();
		type Call = Call;
		type DbWeight = ();
		type Event = ();
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Header = Header;
		type Index = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type OnSetCode = ();
		type Origin = Origin;
		type PalletInfo = PalletInfo;
		type SS58Prefix = ();
		type SystemWeightInfo = ();
		type Version = ();
	}
	parameter_types! {
		pub const ExistentialDeposit: u64 = 1;
		pub const MaxReserves: u32 = 50;
	}
	impl pallet_balances::Config for Runtime {
		type AccountStore = frame_system::Pallet<Runtime>;
		type Balance = Balance;
		type DustRemoval = ();
		type Event = ();
		type ExistentialDeposit = ExistentialDeposit;
		type MaxLocks = ();
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = ReserveIdentifier;
		type WeightInfo = ();
	}
	impl pallet_utility::Config for Runtime {
		type Call = Call;
		type Event = ();
		type WeightInfo = ();
	}
	parameter_types! {
		pub const ProxyDepositBase: u64 = 1;
		pub const ProxyDepositFactor: u64 = 1;
		pub const MaxProxies: u16 = 4;
		pub const MaxPending: u32 = 2;
		pub const AnnouncementDepositBase: u64 = 1;
		pub const AnnouncementDepositFactor: u64 = 1;
	}
	#[derive(
		Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, MaxEncodedLen,
	)]
	pub enum ProxyType {
		Any,
		JustTransfer,
		JustUtility,
	}
	impl Default for ProxyType {
		fn default() -> Self {
			Self::Any
		}
	}
	impl InstanceFilter<Call> for ProxyType {
		fn filter(&self, c: &Call) -> bool {
			match self {
				ProxyType::Any => true,
				ProxyType::JustTransfer => {
					matches!(c, Call::Balances(pallet_balances::Call::transfer(..)))
				},
				ProxyType::JustUtility => matches!(c, Call::Utility(..)),
			}
		}

		fn is_superset(&self, o: &Self) -> bool {
			self == &ProxyType::Any || self == o
		}
	}
	pub struct BaseFilter;
	impl Contains<Call> for BaseFilter {
		fn contains(c: &Call) -> bool {
			match *c {
				// Remark is used as a no-op call in the benchmarking
				Call::System(SystemCall::remark(_)) => true,
				Call::System(_) => false,
				_ => true,
			}
		}
	}
	impl pallet_proxy::Config for Runtime {
		type AnnouncementDepositBase = AnnouncementDepositBase;
		type AnnouncementDepositFactor = AnnouncementDepositFactor;
		type Call = Call;
		type CallHasher = BlakeTwo256;
		type Currency = Balances;
		type Event = ();
		type MaxPending = MaxPending;
		type MaxProxies = MaxProxies;
		type ProxyDepositBase = ProxyDepositBase;
		type ProxyDepositFactor = ProxyDepositFactor;
		type ProxyType = ProxyType;
		type WeightInfo = ();
	}

	parameter_types! {
		pub const CreateClassDeposit: Balance = 200;
		pub const CreateTokenDeposit: Balance = 100;
		pub const DataDepositPerByte: Balance = 10;
		pub const NftPalletId: PalletId = PalletId(*b"aca/aNFT");
		pub MaxAttributesBytes: u32 = 2048;
	}

	impl crate::Config for Runtime {
		type CreateClassDeposit = CreateClassDeposit;
		type CreateTokenDeposit = CreateTokenDeposit;
		type Currency = Balances;
		type DataDepositPerByte = DataDepositPerByte;
		type Event = ();
		type MaxAttributesBytes = MaxAttributesBytes;
		type PalletId = NftPalletId;
		type WeightInfo = ();
	}

	parameter_types! {
		pub const MaxClassMetadata: u32 = 1024;
		pub const MaxTokenMetadata: u32 = 1024;
	}

	impl orml_nft::Config for Runtime {
		type ClassData = ClassData<Balance>;
		type ClassId = u32;
		type MaxClassMetadata = MaxClassMetadata;
		type MaxTokenMetadata = MaxTokenMetadata;
		type TokenData = TokenData<Balance>;
		type TokenId = u64;
	}

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
	type Block = frame_system::mocking::MockBlock<Runtime>;

	frame_support::construct_runtime!(
		pub enum Runtime where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Utility: pallet_utility::{Pallet, Call, Event},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
			OrmlNFT: orml_nft::{Pallet, Storage, Config<T>},
			NFT: nft::{Pallet, Call, Event<T>},
		}
	);

	use frame_system::Call as SystemCall;

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

#[cfg(test)]
mod tests {
	use frame_benchmarking::impl_benchmark_test_suite;

	use super::{mock::*, *};

	impl_benchmark_test_suite!(Pallet, super::new_test_ext(), super::Runtime,);
}
