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
    Event, OrderModule, ExtBuilder, Origin, System, Tokens, ALICE, BOB,
};
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

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

#[test]
fn test_make_order_should_work() {
    new_test_ext().execute_with(|| {

    });
}
fn test_make_order_should_fail() {
    new_test_ext().execute_with(|| {

    });
}

#[test]
fn test_take_order_should_work() {
    new_test_ext().execute_with(|| {
       
    });
}
fn test_take_order_should_fail() {
    new_test_ext().execute_with(|| {
       
    });
}

#[test]
fn test_cancel_order_should_work() {
    new_test_ext().execute_with(|| {
        
    });
}
fn test_cancel_order_should_fail() {
    new_test_ext().execute_with(|| {
        
    });
}