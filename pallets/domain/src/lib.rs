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

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use codec::{Encode, Decode};
	use frame_support::{
		dispatch::{PostDispatchInfo, DispatchResultWithPostInfo},
		pallet_prelude::*,
		weights::GetDispatchInfo,
		RuntimeDebug,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use sp_runtime::traits::Dispatchable;

	#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug)]
	pub struct DomainAddress<AccountId> {
		native: AccountId,
		relay: Option<AccountId>,
		ethereum: Vec<u8>,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The overarching call type.
		type Call: Parameter + Dispatchable<Origin=Self::Origin, PostInfo=PostDispatchInfo> + GetDispatchInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn domains)]
	pub(super) type Domains<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn domain_addresses)]
	pub(super) type DomainAddresses<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, DomainAddress<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub(super) type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, T::AccountId, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DomainSet(T::AccountId, Vec<u8>),
		Sent(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidTarget,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(0)]
		pub(super) fn set_domain(
			origin: OriginFor<T>,
			domain: Vec<u8>,
			ethereum: Vec<u8>,
			relay: Option<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			<DomainAddresses<T>>::insert(&domain, DomainAddress {
				native: who.clone(),
				relay,
				ethereum,
			});
			<Domains<T>>::insert(&who, &domain);
			<Accounts<T>>::insert(&domain, &who);

			Self::deposit_event(Event::DomainSet(who, domain));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub(super) fn send(
			origin: OriginFor<T>,
			target: T::AccountId,
			target_domain: Vec<u8>,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let domain = <Domains<T>>::get(&target);
			let account = <Accounts<T>>::get(&target_domain);
			ensure!(&target == &account, Error::<T>::InvalidTarget);

			call.dispatch(frame_system::RawOrigin::Signed(who.clone()).into())
				.map(|_| ()).map_err(|e| e.error)?;

			Self::deposit_event(Event::Sent(who, domain));

			Ok(().into())
		}
	}
}
