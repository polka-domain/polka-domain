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

use frame_support::{assert_noop, assert_ok};

use crate::{mock::*, Error};

#[test]
fn test_register() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_noop!(
			DomainModule::register(
				Origin::signed(1),
				vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
				Some(vec![2]),
				Some(vec![3]),
				Some(vec![4]),
				Some(vec![5]),
			),
			Error::<Runtime>::InvalidDomainLength
		);

		assert_ok!(DomainModule::register(
			Origin::signed(1),
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5]),
		));

		assert_noop!(
			DomainModule::register(
				Origin::signed(1),
				vec![1],
				Some(vec![2]),
				Some(vec![3]),
				Some(vec![4]),
				Some(vec![5])
			),
			Error::<Runtime>::DomainMustExist
		);

		let event = Event::DomainModule(crate::Event::DomainRegistered(
			1,
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5]),
			1,
			crate::DomainInfos::<Runtime>::get(vec![1]).nft_token,
		));
		assert_eq!(last_event(), event);

		assert_eq!(crate::Domains::<Runtime>::get(1), vec![1]);
		assert_eq!(
			crate::DomainInfos::<Runtime>::get(vec![1]),
			crate::DomainInfo {
				native: 1,
				bitcoin: Some(vec![2]),
				ethereum: Some(vec![3]),
				polkadot: Some(0), //todo only vec<u8> 32 can convert to AccountId
				kusama: Some(0),   //todo only vec<u8> 32 can convert to AccountId
				deposit: 1,
				nft_token: (0, 1)
			}
		);
	});
}

#[test]
fn deregister() {
	new_test_ext().execute_with(|| {
		assert_ok!(DomainModule::register(
			Origin::signed(1),
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5])
		));

		let event = Event::DomainModule(crate::Event::DomainRegistered(
			1,
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5]),
			1,
			crate::DomainInfos::<Runtime>::get(vec![1]).nft_token,
		));
		assert_eq!(last_event(), event);

		let nft_token_deregistered = crate::DomainInfos::<Runtime>::get(vec![1]).nft_token;
		assert_ok!(DomainModule::deregister(Origin::signed(1), vec![1]));
		let event = Event::DomainModule(crate::Event::DomainDeregistered(
			1,
			vec![1],
			nft_token_deregistered,
		));
		assert_eq!(last_event(), event);
		assert_eq!(crate::Domains::<Runtime>::get(1), Vec::<u8>::new());
		assert_eq!(crate::DomainInfos::<Runtime>::get(vec![1]), Default::default());
	});
}

#[test]
fn send() {
	new_test_ext().execute_with(|| {
		assert_ok!(DomainModule::register(
			Origin::signed(1),
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5])
		));

		let event = Event::DomainModule(crate::Event::DomainRegistered(
			1,
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5]),
			1,
			crate::DomainInfos::<Runtime>::get(vec![1]).nft_token,
		));
		assert_eq!(last_event(), event);

		let call = Box::new(Call::Balances(BalancesCall::transfer(1, 100)));
		assert_ok!(DomainModule::send(Origin::signed(2), 1, vec![1], call));

		let event = Event::DomainModule(crate::Event::Sent(2, vec![1]));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn transfer() {
	new_test_ext().execute_with(|| {
		assert_ok!(DomainModule::register(
			Origin::signed(1),
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5])
		));

		let event = Event::DomainModule(crate::Event::DomainRegistered(
			1,
			vec![1],
			Some(vec![2]),
			Some(vec![3]),
			Some(vec![4]),
			Some(vec![5]),
			1,
			crate::DomainInfos::<Runtime>::get(vec![1]).nft_token,
		));
		assert_eq!(last_event(), event);

		assert_ok!(DomainModule::transfer(Origin::signed(1), 2, vec![1]));

		let event = Event::DomainModule(crate::Event::Transfer(
			1,
			2,
			vec![1],
			crate::DomainInfos::<Runtime>::get(vec![1]).nft_token,
		));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn bind_address() {
	new_test_ext().execute_with(|| {
		assert_ok!(DomainModule::register(
			Origin::signed(1),
			vec![1],
			Some(vec![1]),
			Some(vec![1]),
			Some(vec![1]),
			Some(vec![1])
		));

		let event = Event::DomainModule(crate::Event::DomainRegistered(
			1,
			vec![1],
			Some(vec![1]),
			Some(vec![1]),
			Some(vec![1]),
			Some(vec![1]),
			1,
			crate::DomainInfos::<Runtime>::get(vec![1]).nft_token,
		));
		assert_eq!(last_event(), event);
		assert_eq!(crate::DomainInfos::<Runtime>::get(vec![1]).ethereum, Some(vec![1]));

		assert_ok!(DomainModule::bind_address(
			Origin::signed(1),
			vec![1],
			crate::AddressChainType::ETH,
			vec![3]
		));

		let event = Event::DomainModule(crate::Event::BindAddress(
			1,
			vec![1],
			crate::AddressChainType::ETH,
			vec![3],
		));
		assert_eq!(last_event(), event);
		assert_eq!(crate::DomainInfos::<Runtime>::get(vec![1]).ethereum, Some(vec![3]));

		assert_ok!(DomainModule::bind_address(
			Origin::signed(1),
			vec![1],
			crate::AddressChainType::BTC,
			vec![4]
		));
		let event = Event::DomainModule(crate::Event::BindAddress(
			1,
			vec![1],
			crate::AddressChainType::BTC,
			vec![4],
		));
		assert_eq!(last_event(), event);
		assert_eq!(crate::DomainInfos::<Runtime>::get(vec![1]).bitcoin, Some(vec![4]));
	});
}
