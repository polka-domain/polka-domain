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

#![cfg(test)]

use codec::{Decode, Encode};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Filter, InstanceFilter},
	PalletId, RuntimeDebug,
};
use orml_traits::parameter_type_with_key;
use primitives::{Amount, Balance, BlockNumber, CurrencyId, TokenSymbol};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};

use super::*;
use crate as pallet_auction;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

pub type AccountId = AccountId32;
pub type AuctionId = u32;

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
	type Event = Event;
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
}
impl pallet_balances::Config for Runtime {
	type AccountStore = frame_system::Pallet<Runtime>;
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ();
	type WeightInfo = ();
}

impl pallet_utility::Config for Runtime {
	type Call = Call;
	type Event = Event;
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
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
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
			}
			ProxyType::JustUtility => matches!(c, Call::Utility(..)),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		self == &ProxyType::Any || self == o
	}
}
pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
	fn filter(c: &Call) -> bool {
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
	type Event = Event;
	type MaxPending = MaxPending;
	type MaxProxies = MaxProxies;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type ProxyType = ProxyType;
	type WeightInfo = ();
}

pub type NativeCurrency =
	orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type Amount = Amount;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type Event = Event;
	type ExistentialDeposits = ExistentialDeposits;
	type MaxLocks = ();
	type OnDust = ();
	type WeightInfo = ();
}

pub const NATIVE_CURRENCY_ID: CurrencyId = CurrencyId::Token(TokenSymbol::NAME);

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = NATIVE_CURRENCY_ID;
}

impl orml_currencies::Config for Runtime {
	type Event = Event;
	type GetNativeCurrencyId = GetNativeCurrencyId;
	type MultiCurrency = Tokens;
	type NativeCurrency = NativeCurrency;
	type WeightInfo = ();
}

parameter_types! {
	pub const CreateClassDeposit: Balance = 200;
	pub const CreateTokenDeposit: Balance = 100;
	pub const NftPalletId: PalletId = PalletId(*b"pol/aNFT");
}

impl nft::Config for Runtime {
	type CreateClassDeposit = CreateClassDeposit;
	type CreateTokenDeposit = CreateTokenDeposit;
	type Event = Event;
	type PalletId = NftPalletId;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxClassMetadata: u32 = 256;
	pub const MaxTokenMetadata: u32 = 256;
}

impl orml_nft::Config for Runtime {
	type ClassData = nft::ClassData<Balance>;
	type ClassId = u32;
	type MaxClassMetadata = MaxClassMetadata;
	type MaxTokenMetadata = MaxTokenMetadata;
	type TokenData = nft::TokenData<Balance>;
	type TokenId = u64;
}

parameter_types! {
	pub const MaxAuction: u32 = 3;
}
impl pallet_auction::Config for Runtime {
	type AuctionId = AuctionId;
	type Balance = Balance;
	type ClassData = ();
	type ClassId = u32;
	type Currency = Currency;
	type Event = Event;
	type MaxAuction = MaxAuction;
	type NFT = NFTPallet;
	type TokenData = ();
	type TokenId = u64;
}

use frame_system::Call as SystemCall;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		AuctionModule: pallet_auction::{Pallet, Call, Event<T>},
		OrmlNFT: orml_nft::{Pallet, Storage, Config<T>},
		NFTPallet: nft::{Pallet, Storage, Config<T>, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Currency: orml_currencies::{Pallet, Call, Event<T>},
	}
);

pub const ALICE: AccountId = AccountId::new([1u8; 32]);
pub const BOB: AccountId = AccountId::new([2u8; 32]);
pub const CLASS_ID: <Runtime as orml_nft::Config>::ClassId = 0;
pub const TOKEN_ID: <Runtime as orml_nft::Config>::TokenId = 0;

pub struct ExtBuilder;
impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![
				(ALICE, 1000000000000),
				(BOB, 1000000000000),
				(
					<Runtime as nft::Config>::PalletId::get().into_sub_account(CLASS_ID),
					1000000000000,
				),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events().pop().expect("Event expected").event
}
