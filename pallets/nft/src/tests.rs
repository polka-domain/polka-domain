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

use frame_support::{assert_noop, assert_ok, traits::Currency};
use mock::{Event, *};
use orml_nft::TokenInfo;
use primitives::Balance;
use sp_runtime::{traits::BlakeTwo256, ArithmeticError};
use sp_std::convert::TryInto;

use super::*;

fn free_balance(who: &AccountId) -> Balance {
	<Runtime as pallet_proxy::Config>::Currency::free_balance(who)
}

fn reserved_balance(who: &AccountId) -> Balance {
	<Runtime as pallet_proxy::Config>::Currency::reserved_balance(who)
}

fn class_id_account() -> AccountId {
	<Runtime as Config>::PalletId::get().into_sub_account(CLASS_ID)
}

#[test]
fn create_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), vec![1], Default::default()));
		let event = Event::NFTModule(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			<Runtime as Config>::CreateClassDeposit::get() + Proxy::deposit(1u32)
		);
	});
}

#[test]
fn create_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			NFTModule::create_class(
				Origin::signed(BOB),
				vec![1],
				Properties(ClassProperty::Transferable | ClassProperty::Burnable)
			),
			pallet_balances::Error::<Runtime, _>::InsufficientBalance
		);
	});
}

#[test]
fn mint_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		let event = Event::NFTModule(crate::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				2 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![2], 2));
		let event =
			Event::NFTModule(crate::Event::MintedToken(class_id_account(), BOB, CLASS_ID, 2));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			<Runtime as Config>::CreateClassDeposit::get() + Proxy::deposit(1u32)
		);

		assert_eq!(reserved_balance(&BOB), 2 * <Runtime as Config>::CreateTokenDeposit::get());

		assert_eq!(
			orml_nft::Pallet::<Runtime>::tokens(0, 0).unwrap(),
			TokenInfo {
				metadata: vec![2].try_into().unwrap(),
				owner: BOB,
				data: TokenData { deposit: <Runtime as Config>::CreateTokenDeposit::get() }
			}
		);

		assert_eq!(
			orml_nft::Pallet::<Runtime>::tokens(0, 1).unwrap(),
			TokenInfo {
				metadata: vec![2].try_into().unwrap(),
				owner: BOB,
				data: TokenData { deposit: <Runtime as Config>::CreateTokenDeposit::get() }
			}
		);
	});
}

#[test]
fn mint_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_noop!(
			NFTModule::mint(Origin::signed(ALICE), BOB, CLASS_ID_NOT_EXIST, vec![1], 2),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			NFTModule::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 0),
			Error::<Runtime>::InvalidQuantity
		);

		assert_noop!(
			NFTModule::mint(Origin::signed(BOB), BOB, CLASS_ID, vec![1], 2),
			Error::<Runtime>::NoPermission
		);

		orml_nft::NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
			*id = <Runtime as orml_nft::Config>::TokenId::max_value()
		});
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				2 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_noop!(
			NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 2),
			orml_nft::Error::<Runtime>::NoAvailableTokenId
		);
	});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				2 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 2));

		assert_eq!(reserved_balance(&BOB), 2 * <Runtime as Config>::CreateTokenDeposit::get());

		assert_ok!(NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)));
		let event =
			Event::NFTModule(crate::Event::TransferredToken(BOB, ALICE, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);

		assert_eq!(reserved_balance(&BOB), 1 * <Runtime as Config>::CreateTokenDeposit::get());
		assert_eq!(reserved_balance(&ALICE), 1 * <Runtime as Config>::CreateTokenDeposit::get());

		assert_ok!(NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)));
		let event =
			Event::NFTModule(crate::Event::TransferredToken(ALICE, BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);

		assert_eq!(reserved_balance(&BOB), 2 * <Runtime as Config>::CreateTokenDeposit::get());
		assert_eq!(reserved_balance(&ALICE), 0);
	});
}

#[test]
fn transfer_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				1 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));
		assert_noop!(
			NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID_NOT_EXIST, TOKEN_ID)),
			Error::<Runtime>::ClassIdNotFound
		);
		assert_noop!(
			NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID_NOT_EXIST)),
			Error::<Runtime>::TokenIdNotFound
		);
		assert_noop!(
			NFTModule::transfer(Origin::signed(ALICE), BOB, (CLASS_ID, TOKEN_ID)),
			orml_nft::Error::<Runtime>::NoPermission
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), vec![1], Default::default()));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				1 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));
		assert_noop!(
			NFTModule::transfer(Origin::signed(BOB), ALICE, (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NonTransferable
		);
	});
}

#[test]
fn burn_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				1 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));
		assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		let event = Event::NFTModule(crate::Event::BurnedToken(BOB, CLASS_ID, TOKEN_ID));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			<Runtime as Config>::CreateClassDeposit::get() + Proxy::deposit(1u32)
		);
	});
}

#[test]
fn burn_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				1 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));
		assert_noop!(
			NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID_NOT_EXIST)),
			Error::<Runtime>::TokenIdNotFound
		);

		assert_noop!(
			NFTModule::burn(Origin::signed(ALICE), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NoPermission
		);

		orml_nft::Classes::<Runtime>::mutate(CLASS_ID, |class_info| {
			class_info.as_mut().unwrap().total_issuance = 0;
		});
		assert_noop!(
			NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			ArithmeticError::Overflow,
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(Origin::signed(ALICE), vec![1], Default::default()));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				1 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));
		assert_noop!(
			NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)),
			Error::<Runtime>::NonBurnable
		);
	});
}

#[test]
fn burn_with_remark_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				1 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));

		let remark = "remark info".as_bytes().to_vec();
		let remark_hash = BlakeTwo256::hash(&remark[..]);
		assert_ok!(NFTModule::burn_with_remark(Origin::signed(BOB), (CLASS_ID, TOKEN_ID), remark));
		let event = Event::NFTModule(crate::Event::BurnedTokenWithRemark(
			BOB,
			CLASS_ID,
			TOKEN_ID,
			remark_hash,
		));
		assert_eq!(last_event(), event);

		assert_eq!(
			reserved_balance(&class_id_account()),
			<Runtime as Config>::CreateClassDeposit::get() + Proxy::deposit(1u32)
		);
	});
}

#[test]
fn destroy_class_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_ok!(Balances::deposit_into_existing(
			&class_id_account(),
			1 * <Runtime as Config>::CreateTokenDeposit::get()
		)); // + 100
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));
		assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));
		assert_ok!(NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID, ALICE));
		let event = Event::NFTModule(crate::Event::DestroyedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

		assert_eq!(free_balance(&class_id_account()), 0);
		assert_eq!(reserved_balance(&class_id_account()), 0);
		assert_eq!(free_balance(&ALICE), 100000);
		assert_eq!(free_balance(&BOB), <Runtime as Config>::CreateTokenDeposit::get());
	});
}

#[test]
fn destroy_class_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTModule::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));
		assert_eq!(
			Balances::deposit_into_existing(
				&class_id_account(),
				1 * <Runtime as Config>::CreateTokenDeposit::get()
			)
			.is_ok(),
			true
		);
		assert_ok!(NFTModule::mint(Origin::signed(class_id_account()), BOB, CLASS_ID, vec![1], 1));
		assert_noop!(
			NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID_NOT_EXIST, BOB),
			Error::<Runtime>::ClassIdNotFound
		);

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(BOB), CLASS_ID, BOB),
			Error::<Runtime>::NoPermission
		);

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB),
			Error::<Runtime>::CannotDestroyClass
		);

		assert_ok!(NFTModule::burn(Origin::signed(BOB), (CLASS_ID, TOKEN_ID)));

		assert_noop!(
			NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID, BOB),
			pallet_proxy::Error::<Runtime>::NotFound
		);

		assert_ok!(NFTModule::destroy_class(Origin::signed(class_id_account()), CLASS_ID, ALICE));
	});
}
