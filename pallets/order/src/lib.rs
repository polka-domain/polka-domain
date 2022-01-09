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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use frame_support::RuntimeDebug;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
pub use pallet::*;
use primitives::{CurrencyId, NFT};
use sp_runtime::traits::{AtLeast32BitUnsigned, One, Saturating};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
pub struct PoolDetails<AccountId, Balance, ClassId, TokenId> {
	maker: AccountId,
	taker: Option<AccountId>,
	token0: (ClassId, TokenId),
	token1: CurrencyId,
	total1: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The units in which we record balances.
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize;

		/// The arithmetic type of order identifier.
		type OrderId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

		/// The currency mechanism.
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = CurrencyId, Balance = Self::Balance>
			+ MultiReservableCurrency<Self::AccountId>;

		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;

		// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;

		/// The class properties type
		type ClassData: Parameter + Member + MaybeSerializeDeserialize;

		/// The token properties type
		type TokenData: Parameter + Member + MaybeSerializeDeserialize;

		/// The NFT mechanism
		type NFT: NFT<
			Self::AccountId,
			ClassId = Self::ClassId,
			TokenId = Self::TokenId,
			Balance = Self::Balance,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Next id of an order
	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub(super) type NextOrderId<T: Config> = StorageValue<_, T::OrderId>;

	/// Details of an order.
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(super) type Order<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::OrderId,
		PoolDetails<T::AccountId, T::Balance, T::ClassId, T::TokenId>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrderCreated(T::OrderId, T::AccountId, (T::ClassId, T::TokenId), CurrencyId, T::Balance),
		OrderSwapped(T::OrderId, T::AccountId, T::Balance),
		OrderCancelled(T::OrderId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidCreator,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1000)]
		pub fn make_order(
			origin: OriginFor<T>,
			token0: (T::ClassId, T::TokenId),
			token1: CurrencyId,
			total1: T::Balance,
		) -> DispatchResult {
			let maker = ensure_signed(origin)?;
			let order_id = NextOrderId::<T>::get().unwrap_or_default();

			T::NFT::reserve(&maker, token0)?;

			Order::<T>::insert(
				order_id,
				PoolDetails { maker: maker.clone(), taker: None, token0, token1, total1 },
			);
			NextOrderId::<T>::put(order_id.saturating_add(One::one()));

			Self::deposit_event(Event::OrderCreated(order_id, maker, token0, token1, total1));

			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn cancel_order(origin: OriginFor<T>, order_id: T::OrderId) -> DispatchResult {
			let maker = ensure_signed(origin)?;
			let order = Order::<T>::get(order_id);
			ensure!(maker == order.maker, Error::<T>::InvalidCreator);

			T::NFT::unreserve(&order.maker, order.token0)?;

			Order::<T>::remove(order_id);

			Self::deposit_event(Event::OrderCancelled(order_id));

			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn take_order(
			origin: OriginFor<T>,
			order_id: T::OrderId,
			amount1: T::Balance,
		) -> DispatchResult {
			let taker = ensure_signed(origin)?;

			Order::<T>::try_mutate(order_id, |order| -> DispatchResult {
				T::NFT::unreserve(&order.maker, order.token0)?;
				T::NFT::transfer(&order.maker, &taker, order.token0)?;
				T::Currency::transfer(order.token1, &taker, &order.maker, amount1)?;
				order.taker = Some(taker.clone());

				Self::deposit_event(Event::OrderSwapped(order_id, taker, amount1));
				Ok(())
			})?;

			Ok(())
		}
	}
}
