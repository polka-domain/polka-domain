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

use super::*;

use crate as pallet_domain_registrar;
use sp_core::H256;
use frame_support::parameter_types;
use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header,
};
use frame_system as system;
use primitives::{Balance};


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
		pub enum Runtime where
				Block = Block,
				NodeBlock = Block,
				UncheckedExtrinsic = UncheckedExtrinsic,
		{
				System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
				Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
				DomainModule: pallet_domain_registrar::{Pallet, Call, Storage, Event<T>},
		}
);

parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const SS58Prefix: u8 = 42;
}

impl system::Config for Runtime {
		type BaseCallFilter = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Runtime>;
	type MaxLocks = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const DomainDeposit: u32 = 1;
	pub const MaxDomainLen: u32 = 10;
}

impl pallet_domain_registrar::Config for Runtime {
	type Event = Event;
	type DomainDeposit = DomainDeposit;
	type MaxDomainLen = MaxDomainLen;
	type Currency = Balances;
	type Call = Call;

}

pub type BalancesCall = pallet_balances::Call<Runtime>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap()
		.into();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(1, 100000), (2, 100000)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn last_event() -> Event {
	frame_system::Pallet::<Runtime>::events()
		.pop()
		.expect("Event expected")
		.event
}