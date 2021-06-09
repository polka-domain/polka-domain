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
    Event, AuctionModule, ExtBuilder, NFTPallet, Proxy, Origin, System, Tokens, ALICE, BOB,
    CLASS_ID, TOKEN_ID
};
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use primitives::{Balance, TokenSymbol};
use nft::{Properties, ClassProperty};
use sp_runtime::{
	traits::{AccountIdConversion},
};
use frame_support::traits::OnFinalize;

const DURATION: u64 = 10;
const WAITING_BEGIN_BLOCK_AFTER_CREATED: u64 = 1;

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
fn test_create_auction_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));

        let event = Event::nft(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

        assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
        let event = Event::nft(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

        assert_noop!(
            AuctionModule::create_auction(
                Origin::signed(ALICE),
                (CLASS_ID, TOKEN_ID),
                CurrencyId::Token(TokenSymbol::NAME),
                1,
                0
            ),
            Error::<Runtime>::InvalidDuration
        );

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(0, ALICE,
            (CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10,
            2,
            2 + 10
        ));
		assert_eq!(last_event(), event);
    });
}

#[test]
fn test_create_auction_should_fail() {
    new_test_ext().execute_with(|| {
        assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));

        let event = Event::nft(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

        assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			5
		));
        let event = Event::nft(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 5));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, 0),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            0, 
            ALICE,
            (CLASS_ID, 0),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10,
            2,
            2 + 10
        ));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, 1),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            1, 
            ALICE,
            (CLASS_ID, 1),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10,
            2,
            2 + 10
        ));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, 2),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            2, 
            ALICE,
            (CLASS_ID, 2),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10,
            2,
            2 + 10
        ));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, 3),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            3, 
            ALICE,
            (CLASS_ID, 3),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10,
            2,
            2 + 10
        ));
		assert_eq!(last_event(), event);

        assert_noop!(
            AuctionModule::create_auction(
                Origin::signed(ALICE),
                (CLASS_ID, 4),
                CurrencyId::Token(TokenSymbol::NAME),
                1,
                10
            ),
            Error::<Runtime>::ExceedMaxAuction
        );

        
    });
}

#[test]
fn test_bid_auction_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));

        let event = Event::nft(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

        assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
        let event = Event::nft(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            2,
            DURATION
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            0, 
            ALICE,
            (CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            2,
            DURATION,
            2,
            2 + DURATION
        ));
		assert_eq!(last_event(), event);

        assert_ok!(
            AuctionModule::bid_auction(
                Origin::signed(BOB), 
                0,
                2
            )
        );
        let event = Event::pallet_auction(crate::Event::AuctionBid(0, BOB, 2));
		assert_eq!(last_event(), event);

        let before_alice_balance = free_balance(&ALICE);

        AuctionModule::on_finalize(12);

        assert_eq!(free_balance(&ALICE), before_alice_balance + 2);
        
        let event = Event::pallet_auction(crate::Event::AuctionEnd(0, Some(BOB), Some(2)));
		assert_eq!(last_event(), event);
    });
}

#[test]
fn test_bid_auction_should_fail() {
    new_test_ext().execute_with(|| {
        assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));

        let event = Event::nft(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

        assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
        let event = Event::nft(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            2,
            DURATION
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            0, 
            ALICE,
            (CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            2,
            DURATION,
            2,
            2 + DURATION
        ));
		assert_eq!(last_event(), event);

        assert_noop!(
            AuctionModule::bid_auction(
                Origin::signed(BOB), 
                0,
                1
            ),
            Error::<Runtime>::InvalidBidAmount
        );

        assert_ok!(
            AuctionModule::bid_auction(
                Origin::signed(BOB), 
                0,
                2
            )
        );

        assert_noop!(
            AuctionModule::bid_auction(
                Origin::signed(BOB), 
                0,
                2
            ),
            Error::<Runtime>::InvalidBidAmount
        );


        let now = frame_system::Pallet::<Runtime>::block_number();
        System::set_block_number(now + DURATION + WAITING_BEGIN_BLOCK_AFTER_CREATED + 1);

        assert_noop!(
            AuctionModule::bid_auction(
                Origin::signed(BOB), 
                0,
                3
            ),
            Error::<Runtime>::AuctionExpired
        );
    });
}

#[test]
fn test_cancel_auction_should_work() {
    new_test_ext().execute_with(|| {
        assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));

        let event = Event::nft(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

        assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
        let event = Event::nft(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            0, 
            ALICE,
            (CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            1,
            10,
            2,
            2 + 10
        ));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::cancel_auction(
            Origin::signed(ALICE),
			0
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCancelled(0));
		assert_eq!(last_event(), event);

    });
}

#[test]
fn test_cancel_auction_should_fail() {
    new_test_ext().execute_with(|| {
        assert_ok!(NFTPallet::create_class(
			Origin::signed(ALICE),
			vec![1],
			Properties(ClassProperty::Transferable | ClassProperty::Burnable)
		));

        let event = Event::nft(nft::Event::CreatedClass(class_id_account(), CLASS_ID));
		assert_eq!(last_event(), event);

        assert_ok!(NFTPallet::mint(
			Origin::signed(class_id_account()),
			ALICE,
			CLASS_ID,
			vec![2],
			2
		));
        let event = Event::nft(nft::Event::MintedToken(class_id_account(), ALICE, CLASS_ID, 2));
		assert_eq!(last_event(), event);

        assert_ok!(AuctionModule::create_auction(
            Origin::signed(ALICE),
			(CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            2,
            DURATION
        ));
        let event = Event::pallet_auction(crate::Event::AuctionCreated(
            0, 
            ALICE,
            (CLASS_ID, TOKEN_ID),
            CurrencyId::Token(TokenSymbol::NAME),
            2,
            DURATION,
            2,
            2 + DURATION
        ));
		assert_eq!(last_event(), event);

        assert_noop!(
            AuctionModule::cancel_auction(
                Origin::signed(BOB), 
                0
            ),
            Error::<Runtime>::InvalidCreator
        );

        //add some block
        let now = frame_system::Pallet::<Runtime>::block_number();
        System::set_block_number(now + DURATION);

        assert_noop!(
            AuctionModule::cancel_auction(
                Origin::signed(ALICE), 
                0
            ),
            Error::<Runtime>::AuctionStarted
        );
    });
}