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

use super::*;
pub use crate::mock::{
    Event, OrderModule, ExtBuilder, NFTPallet, Proxy, Origin, System, Tokens, ALICE, BOB,
    CLASS_ID, CLASS_ID_NOT_EXIST, TOKEN_ID, TOKEN_ID_NOT_EXIST
};
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use primitives::{Balance, TokenSymbol};
use nft::{Properties, ClassProperty};
use sp_runtime::{
	traits::{AccountIdConversion},
};

fn new_test_ext() -> sp_io::TestExternalities {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| System::set_block_number(1));
    ext
}

fn events() -> Vec<Event> {
    let evt = System::events()
        .into_iter()
        .map(|evt| evt.event)
        .collect::<Vec<_>>();
    System::reset_events();
    evt
}

fn free_balance(who: &AccountId) -> Balance {
	<Runtime as pallet_proxy::Config>::Currency::free_balance(who)
}

fn reserved_balance(who: &AccountId) -> Balance {
	<Runtime as pallet_proxy::Config>::Currency::reserved_balance(who)
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

        assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			BOB,
			CLASS_ID,
			vec![2],
			2
		));

        assert_ok!(OrderModule::make_order(
            Origin::signed(BOB),
			(CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            1
        ));

    });
}

// #[test]
// fn test_take_order_should_work() {
//     new_test_ext().execute_with(|| {
       
//     });
// }

// #[test]
// fn test_cancel_order_should_work() {
//     new_test_ext().execute_with(|| {
        
//     });
// }
