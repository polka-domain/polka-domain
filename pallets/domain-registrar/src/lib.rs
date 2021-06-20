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
use frame_support::{
	dispatch::PostDispatchInfo, traits::Currency, weights::GetDispatchInfo, PalletId,
};
pub use pallet::*;
use sp_runtime::{
	traits::{Dispatchable, StaticLookup},
	RuntimeDebug,
};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct DomainInfo<AccountId, Balance, ClassId, TokenId> {
	native: AccountId,
	relay: Option<AccountId>,
	ethereum: Vec<u8>,
	deposit: Balance,
	nft_token: (ClassId, TokenId),
}

pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

pub type CreateClassDepositOf<T> = <T as nft::Config>::CreateClassDeposit;
pub type CreateTokenDepositOf<T> = <T as nft::Config>::CreateTokenDeposit;

const DomainPalletId: PalletId = PalletId(*b"domain!!");

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;

	use super::*;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config + orml_nft::Config + nft::Config {
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
		type Call: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo;

		/// The class properties type
		type ClassData: Parameter + Member + MaybeSerializeDeserialize;

		/// The token properties type
		type TokenData: Parameter + Member + MaybeSerializeDeserialize;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn domains)]
	pub(super) type Domains<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn domain_addresses)]
	pub(super) type DomainInfos<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		DomainInfo<T::AccountId, BalanceOf<T>, ClassIdOf<T>, TokenIdOf<T>>,
		ValueQuery,
	>;

	pub type GenesisDomainData<T> = (
		<T as frame_system::Config>::AccountId, // Domain owner
		Vec<u8>,                                // Domain string
		Vec<u8>,                                // Domain ethereum
	);

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance", ClassIdOf<T> = "ClassId", TokenIdOf<T> = "TokenId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DomainRegistered(T::AccountId, Vec<u8>, Vec<u8>, BalanceOf<T>), //todo add tokenid and classid
		DomainDeregistered(T::AccountId, Vec<u8>),                      //todo add tokenid and classid
		Sent(T::AccountId, Vec<u8>),
		Transfer(T::AccountId, T::AccountId, Vec<u8>, Vec<u8>), //todo add tokenid and classid
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub domains: Vec<GenesisDomainData<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { domains: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.domains.iter().for_each(|_item| {
				// let who = &item.0;
				// let owner: T::AccountId = DomainPalletId::get().into_account();
			})
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidDomainLength,
		InvalidTarget,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub(super) fn register(
			origin: OriginFor<T>,
			domain: Vec<u8>,
			ethereum: Vec<u8>,
			relay: Option<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			ensure!(
				domain.len() <= T::MaxDomainLen::get() as usize,
				Error::<T>::InvalidDomainLength
			);
			//todo register create class,
			// mint token

			// let owner: T::AccountId = nft::Pallet::<T>::PalletId::get().into_account();
			// let class_deposit = nft::Pallet::<T>::CreateClassDeposit::get();

			// let proxy_deposit = <pallet_proxy::Pallet<T>>::deposit(1u32);
			// let total_deposit = proxy_deposit.saturating_add(class_deposit);

			//let token_id = orml_nft::Pallet::<T>::next_token_id(0);
			let token_id = 0u32;
			//todo get nft domain class id replace to 0
			nft::Pallet::<T>::mint(
				origin,
				T::Lookup::unlookup(who.clone()),
				0u32.into(),
				domain.clone(),
				1,
			)?;

			let deposit = T::DomainDeposit::get();
			<T as pallet::Config>::Currency::reserve(&who, deposit)?;
			<DomainInfos<T>>::insert(
				&domain,
				DomainInfo {
					native: who.clone(),
					relay,
					ethereum: ethereum.clone(),
					deposit,
					nft_token: (0u32.into(), token_id.into()),
				},
			);
			<Domains<T>>::insert(&who, &domain);

			Self::deposit_event(Event::DomainRegistered(who, domain, ethereum, deposit));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub(super) fn deregister(
			origin: OriginFor<T>,
			domain: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			let domain_info = <DomainInfos<T>>::take(&domain);
			nft::Pallet::<T>::burn(origin, domain_info.nft_token)?;

			<Domains<T>>::remove(&who);
			<T as pallet::Config>::Currency::unreserve(&who, domain_info.deposit);

			Self::deposit_event(Event::DomainDeregistered(who, domain));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub(super) fn send(
			origin: OriginFor<T>,
			target: T::AccountId,
			_target_domain: Vec<u8>,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			let domain = <Domains<T>>::get(&target);

			call.dispatch(frame_system::RawOrigin::Signed(who.clone()).into())
				.map(|_| ())
				.map_err(|e| e.error)?;

			Self::deposit_event(Event::Sent(who, domain));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub(super) fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			domain: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			<DomainInfos<T>>::try_mutate(&domain, |domain_info| -> DispatchResultWithPostInfo {
				nft::Pallet::<T>::transfer(
					origin,
					T::Lookup::unlookup(to.clone()),
					domain_info.nft_token,
				)?;

				<Domains<T>>::remove(&who);
				<Domains<T>>::insert(&to, &domain);

				domain_info.native = to;

				Ok(().into())
			})?;

			Ok(().into())
		}
	}
}
