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
use frame_support::RuntimeDebug;
use orml_traits::{MultiCurrency, MultiReservableCurrency};
pub use pallet::*;
use sp_runtime::traits::{AtLeast32BitUnsigned, Saturating, Zero};
use sp_std::prelude::*;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct PoolDetails<AccountId, Balance, BlockNumber, TokenId> {
	name: Vec<u8>,
	creator: AccountId,
	token0: TokenId,
	token1: TokenId,
	total0: Balance,
	total1: Balance,
	swapped0: Balance,
	swapped1: Balance,
	duration: BlockNumber,
	start_at: BlockNumber,
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

		/// The arithmetic type of pool identifier.
		type PoolId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

		/// The type of token identifier.
		type TokenId: Member + Parameter + Default + Copy + MaybeSerializeDeserialize;

		/// The currency mechanism.
		type Currency: MultiCurrency<Self::AccountId, CurrencyId = Self::TokenId, Balance = Self::Balance>
			+ MultiReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Next id of a pool
	#[pallet::storage]
	#[pallet::getter(fn next_pool_id)]
	pub(super) type NextPoolId<T: Config> = StorageValue<_, T::PoolId>;

	/// Details of a pool.
	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub(super) type Pool<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::PoolId,
		PoolDetails<T::AccountId, T::Balance, T::BlockNumber, T::TokenId>,
		ValueQuery,
	>;

	/// Swap records by a pool and an account.
	#[pallet::storage]
	#[pallet::getter(fn swaps)]
	pub(super) type Swap<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::PoolId,
		Blake2_128Concat,
		T::AccountId,
		(T::Balance, T::Balance),
		ValueQuery,
	>;

	/// The end block number of a pool
	#[pallet::storage]
	#[pallet::getter(fn pool_end_at)]
	pub(super) type PoolEndAt<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Twox64Concat,
		T::PoolId,
		Option<()>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", T::PoolId = "PoolId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		PoolCreated(T::PoolId, T::AccountId),
		PoolSwapped(T::PoolId, T::AccountId),
		PoolClosed(T::PoolId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidDuration,
		PoolExpired,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(
			origin: OriginFor<T>,
			name: Vec<u8>,
			token0: T::TokenId,
			token1: T::TokenId,
			total0: T::Balance,
			total1: T::Balance,
			duration: T::BlockNumber,
		) -> DispatchResult {
			ensure!(duration > Zero::zero(), Error::<T>::InvalidDuration);

			let creator = ensure_signed(origin)?;
			let pool_id = NextPoolId::<T>::get().unwrap();
			let start_at = frame_system::Pallet::<T>::block_number();
			let end_at = start_at.saturating_add(duration);

			T::Currency::reserve(token0, &creator, total0)?;

			Pool::<T>::insert(
				pool_id,
				PoolDetails {
					name,
					creator: creator.clone(),
					token0,
					token1,
					total0,
					total1,
					swapped0: Zero::zero(),
					swapped1: Zero::zero(),
					duration,
					start_at,
				},
			);
			PoolEndAt::<T>::insert(end_at, pool_id, Some(()));
			NextPoolId::<T>::put(pool_id.saturating_add(1u32.into()));

			Self::deposit_event(Event::PoolCreated(pool_id, creator));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn swap(
			origin: OriginFor<T>,
			pool_id: T::PoolId,
			amount1: T::Balance,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			Pool::<T>::try_mutate(pool_id, |pool| -> DispatchResult {
				let now = frame_system::Pallet::<T>::block_number();
				ensure!(now < pool.start_at.saturating_add(pool.duration), Error::<T>::PoolExpired);

				let amount0: T::Balance = amount1.saturating_mul(pool.total0) / pool.total1;
				pool.swapped0 = pool.swapped0.saturating_add(amount0);
				pool.swapped1 = pool.swapped1.saturating_add(amount1);

				T::Currency::unreserve(pool.token0, &pool.creator, amount0);
				T::Currency::transfer(pool.token0, &pool.creator, &buyer, amount0)?;
				T::Currency::transfer(pool.token1, &buyer, &pool.creator, amount1)?;

				Swap::<T>::try_mutate(pool_id, &buyer, |swap| -> DispatchResult {
					swap.0 = swap.0.saturating_add(amount0);
					swap.1 = swap.1.saturating_add(amount1);
					Ok(())
				})?;

				Self::deposit_event(Event::PoolSwapped(pool_id, buyer));
				Ok(())
			})?;

			Ok(())
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(now: T::BlockNumber) {
			Self::on_finalize(now);
		}
	}

	impl<T: Config> Pallet<T> {
		fn on_finalize(now: T::BlockNumber) {
			for (pool_id, _) in PoolEndAt::<T>::drain_prefix(&now) {
				let pool = Pool::<T>::get(pool_id);
				let un_swapped0 = pool.total0.saturating_sub(pool.swapped0);
				if un_swapped0 > Zero::zero() {
					T::Currency::unreserve(pool.token0, &pool.creator, un_swapped0);
				}
				Self::deposit_event(Event::PoolClosed(pool_id));
			}
		}
	}
}
