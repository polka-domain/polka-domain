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

use codec::{Encode, Decode};
use frame_support::{
	dispatch::PostDispatchInfo,
	weights::GetDispatchInfo,
	traits::Currency,
};
use sp_runtime::{traits::Dispatchable, RuntimeDebug};
use sp_std::prelude::*;

pub use pallet::*;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct DomainInfo<AccountId, Balance> {
	native: AccountId,
	relay: Option<AccountId>,
	ethereum: Vec<u8>,
	deposit: Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::ReservableCurrency;
	use super::*;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {

		#[pallet::constant]
		/// The deposit to be paid to register a domain.
		type DomainDeposit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		/// The deposit to be paid to register a domain.
		type MaxDomainLen: Get<u32>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The system's currency for domain payment.
		type Currency: ReservableCurrency<Self::AccountId>;

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
	pub(super) type DomainInfos<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, DomainInfo<T::AccountId, BalanceOf<T>>, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DomainRegistered(T::AccountId, Vec<u8>),
		DomainDeregistered(T::AccountId, Vec<u8>),
		Sent(T::AccountId, Vec<u8>),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidDomainLength,
		InvalidTarget,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T:Config> Pallet<T> {
		#[pallet::weight(0)]
		pub(super) fn register(
			origin: OriginFor<T>,
			domain: Vec<u8>,
			ethereum: Vec<u8>,
			relay: Option<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(domain.len() <= T::MaxDomainLen::get() as usize, Error::<T>::InvalidDomainLength);
			let deposit = T::DomainDeposit::get();
			T::Currency::reserve(&who, deposit)?;
			<DomainInfos<T>>::insert(&domain, DomainInfo {
				native: who.clone(),
				relay,
				ethereum,
				deposit,
			});
			<Domains<T>>::insert(&who, &domain);

			Self::deposit_event(Event::DomainRegistered(who, domain));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub(super) fn deregister(
			origin: OriginFor<T>,
			domain: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let domain_info = <DomainInfos<T>>::take(&domain);
			<Domains<T>>::remove(&who);
			T::Currency::reserve(&who, domain_info.deposit)?;

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

			call.dispatch(frame_system::RawOrigin::Signed(who.clone()).into())
				.map(|_| ()).map_err(|e| e.error)?;

			Self::deposit_event(Event::Sent(who, domain));

			Ok(().into())
		}
	}
}
