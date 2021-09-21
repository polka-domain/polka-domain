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
use nft::{ClassProperty, Properties};
use primitives::{Balance, TokenSymbol};
use sp_runtime::traits::AccountIdConversion;

use super::*;
pub use crate::mock::{
	Event, ExtBuilder, NFTPallet, OrderModule, Origin, Proxy, System, Tokens, ALICE, BOB, CLASS_ID,
	TOKEN_ID,
};
use crate::{mock::*, Error};

fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = ExtBuilder::default().build();
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn free_balance(who: &AccountId) -> Balance {
	<Runtime as pallet_proxy::Config>::Currency::free_balance(who)
}

fn class_id_account() -> AccountId {
	<Runtime as nft::Config>::PalletId::get().into_sub_account(CLASS_ID)
}

#[test]
fn test_make_order_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));

		let event = Event::NFTPallet(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
		let event =
			Event::NFTPallet(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

		assert_ok!(OrderModule::make_order(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			CurrencyId::Token(TokenSymbol::NAME),
			1
		));
		let event = Event::OrderModule(crate::Event::OrderCreated(
			0,
			ALICE,
			(CLASS_ID, TOKEN_ID),
			CurrencyId::Token(TokenSymbol::NAME),
			1,
		));
		assert_eq!(last_event(), event);
	});
}

#[test]
fn test_take_order_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		let event = Event::NFTPallet(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
		let event =
			Event::NFTPallet(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

		assert_ok!(OrderModule::make_order(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			CurrencyId::Token(TokenSymbol::NAME),
			1
		));
		let event = Event::OrderModule(crate::Event::OrderCreated(
			0,
			ALICE,
			(CLASS_ID, TOKEN_ID),
			CurrencyId::Token(TokenSymbol::NAME),
			1,
		));
		assert_eq!(last_event(), event);

		let before_alice_balance = free_balance(&ALICE);
		let before_bob_balance = free_balance(&BOB);

		assert_ok!(OrderModule::take_order(Origin::signed(BOB), 0, 1));
		let event = Event::OrderModule(crate::Event::OrderSwapped(0, BOB, 1));
		assert_eq!(last_event(), event);

		assert_eq!(free_balance(&ALICE), before_alice_balance + 1);
		assert_eq!(free_balance(&BOB), before_bob_balance - 1);
	});
}

#[test]
fn test_cancel_order_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		let event = Event::NFTPallet(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
		let event =
			Event::NFTPallet(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

		assert_ok!(OrderModule::make_order(
			Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
			CurrencyId::Token(TokenSymbol::NAME),
			1
		));
		let event = Event::OrderModule(crate::Event::OrderCreated(
			0,
			ALICE,
			(CLASS_ID, TOKEN_ID),
			CurrencyId::Token(TokenSymbol::NAME),
			1,
		));
		assert_eq!(last_event(), event);

		assert_noop!(
			OrderModule::cancel_order(Origin::signed(BOB), 0),
			Error::<Runtime>::InvalidCreator
		);

		assert_ok!(OrderModule::cancel_order(Origin::signed(ALICE), 0));
		let event = Event::OrderModule(crate::Event::OrderCancelled(0));
		assert_eq!(last_event(), event);
	});
}
