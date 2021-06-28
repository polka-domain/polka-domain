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
use frame_support::{dispatch::PostDispatchInfo, traits::Currency, weights::GetDispatchInfo};
pub use pallet::*;
use sp_runtime::{traits::Dispatchable, RuntimeDebug};
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct DomainInfo<AccountId, Balance> {
	native: AccountId,
	relay: Option<AccountId>,
	ethereum: Vec<u8>,
	deposit: Balance,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum ChainType {
	BTC,
	ETH,
	DOT,
	DOGE,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;

	use super::*;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
		type Call: Parameter
			+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>;

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
		DomainInfo<T::AccountId, BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DomainRegistered(T::AccountId, Vec<u8>, Vec<u8>, BalanceOf<T>),
		DomainDeregistered(T::AccountId, Vec<u8>),
		Sent(T::AccountId, Vec<u8>),
		Transfer(T::AccountId, T::AccountId, Vec<u8>, Vec<u8>), //todo add tokenid and classid
		BindAddress(T::AccountId, Vec<u8>, ChainType, Vec<u8>),
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
				let who = &_item.0;
				let next_id = orml_nft::Pallet::<T>::next_class_id(); //todo just use one class id
				let owner: T::AccountId =
					<T as nft::Config>::PalletId::get().into_sub_account(next_id);
				let class_deposit = <T as nft::Config>::CreateClassDeposit::get();

				let proxy_deposit = <pallet_proxy::Pallet<T>>::deposit(1u32);
				let total_deposit = proxy_deposit.saturating_add(class_deposit);

				// ensure enough token for proxy deposit + class deposit
				<T as pallet_proxy::Config>::Currency::transfer(
					&who,
					&owner,
					total_deposit,
					KeepAlive,
				)
				.expect("Create class: transfer cannot fail while building genesis");

				<T as pallet_proxy::Config>::Currency::reserve(&owner, class_deposit)
					.expect("Create class: reserve  cannot fail while building genesis");

				// owner add proxy delegate to origin
				<pallet_proxy::Pallet<T>>::add_proxy_delegate(
					&owner,
					who.clone(),
					Default::default(),
					Zero::zero(),
				)
				.expect("Create class: add_proxy_delegate  cannot fail while building genesis");

				let properties = nft::Properties(
					nft::ClassProperty::Transferable | nft::ClassProperty::Burnable,
				);

				let data = nft::ClassData { deposit: class_deposit, properties };

				let class_id = orml_nft::Pallet::<T>::create_class(
					&owner,
					br#"domain-nft-class"#.to_vec(),
					data,
				)
				.expect("Create class:  create_class cannot fail while building genesis");

				// mint nft
				let to = &_item.0;
				let class_info = orml_nft::Pallet::<T>::classes(class_id)
					.expect("Token mint: get class info cannot fail while building genesis");
				if owner != class_info.owner {
					let e: Result<i8, &str> = Err("Error::<T>::NoPermission");
					e.expect("Token mint: Permission cannot fail while building genesis");
				}
				let deposit = <T as nft::Config>::CreateTokenDeposit::get();
				let total_deposit = deposit.saturating_mul(1u32.into());

				<T as pallet_proxy::Config>::Currency::transfer(
					&who,
					&to,
					total_deposit,
					KeepAlive,
				)
				.expect("Token mint: transfer cannot fail while building genesis");
				<T as pallet_proxy::Config>::Currency::reserve(&to, total_deposit)
					.expect("Token mint: reserve  cannot fail while building genesis");

				let token_data = nft::TokenData { deposit };
				orml_nft::Pallet::<T>::mint(
					&to,
					class_id,
					_item.1.to_vec(), // domain string
					token_data.clone(),
				)
				.expect("Token mint cannot fail during genesis");
			})
		}
	}

	#[cfg(feature = "std")]
	impl<T: Config> GenesisConfig<T> {
		/// Direct implementation of `GenesisBuild::build_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn build_storage(&self) -> Result<sp_runtime::Storage, String> {
			<Self as frame_support::traits::GenesisBuild<T>>::build_storage(self)
		}

		/// Direct implementation of `GenesisBuild::assimilate_storage`.
		///
		/// Kept in order not to break dependency.
		pub fn assimilate_storage(&self, storage: &mut sp_runtime::Storage) -> Result<(), String> {
			<Self as frame_support::traits::GenesisBuild<T>>::assimilate_storage(self, storage)
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
			let who = ensure_signed(origin)?;

			ensure!(
				domain.len() <= T::MaxDomainLen::get() as usize,
				Error::<T>::InvalidDomainLength
			);
			// mint token
			//todo get nft domain class id replace to 0
			let token_id = orml_nft::Pallet::<T>::next_token_id(T::NftClassID::get());

			// print!("#### call is Call {:?}", Call::register(0));
			// let call_mint = Box::new(Call::NFT(NFTCall::mint(
			// 	T::Lookup::unlookup(who.clone()),
			// 	0u32.into(),
			// 	domain.clone(),
			// 	1,
			// )));

			// pallet_proxy::Pallet::<T>::proxy(
			// 	origin.clone(),
			// 	<T as nft::Config>::PalletId::get().into_account(),
			// 	None,
			// 	call_mint,
			// );

			let deposit = T::DomainDeposit::get();
			T::Currency::reserve(&who, deposit)?;
			<DomainInfos<T>>::insert(
				&domain,
				DomainInfo { native: who.clone(), relay, ethereum: ethereum.clone(), deposit },
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
			let who = ensure_signed(origin)?;

			let domain_info = <DomainInfos<T>>::take(&domain);
			//print!("#### burn domain_info.nft_token  {:?}", domain_info.nft_token);
			nft::Pallet::<T>::burn(origin, domain_info.nft_token)?;

			<Domains<T>>::remove(&who);
			T::Currency::unreserve(&who, domain_info.deposit);

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

		#[pallet::weight(0)]
		pub(super) fn bind_address(
			origin: OriginFor<T>,
			domain: Vec<u8>,
			chain_type: ChainType,
			address: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			if Domains::<T>::contains_key(&who) {
				if domain == <Domains<T>>::get(&who) {
					DomainInfos::<T>::try_mutate(&domain, |domain_info| -> DispatchResult {
						match chain_type {
							ChainType::ETH => {
								domain_info.ethereum = address;
							}
							ChainType::DOT => {}
							ChainType::DOGE => {}
							_ => {}
						}
						Self::deposit_event(Event::BindAddress(
							who,
							domain.clone(),
							chain_type,
							vec![3],
						));

						Ok(())
					})?;
				}
			}

			Ok(().into())
		}
	}
}
