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
	dispatch::PostDispatchInfo,
	traits::{Currency, ExistenceRequirement::KeepAlive},
	weights::GetDispatchInfo,
};
pub use pallet::*;
use sp_runtime::{
	traits::{AccountIdConversion, Dispatchable, Saturating, StaticLookup, Zero},
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
	bitcoin: Option<Vec<u8>>,
	ethereum: Option<Vec<u8>>,
	polkadot: Option<AccountId>,
	kusama: Option<AccountId>,
	deposit: Balance,
	nft_token: (ClassId, TokenId),
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum AddressChainType {
	BTC,
	ETH,
	DOT,
	KSM,
}

pub type TokenIdOf<T> = <T as orml_nft::Config>::TokenId;
pub type ClassIdOf<T> = <T as orml_nft::Config>::ClassId;

pub type CreateClassDepositOf<T> = <T as nft::Config>::CreateClassDeposit;
pub type CreateTokenDepositOf<T> = <T as nft::Config>::CreateTokenDeposit;

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

		#[pallet::constant]
		/// The deposit to be paid to register a domain.
		type NftClassID: Get<ClassIdOf<Self>>;

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
		DomainRegistered(
			T::AccountId,
			Vec<u8>,
			Option<Vec<u8>>,
			Option<Vec<u8>>,
			Option<Vec<u8>>,
			Option<Vec<u8>>,
			BalanceOf<T>,
			(T::ClassId, T::TokenId),
		), /* todo add tokenid and classid */
		DomainDeregistered(T::AccountId, Vec<u8>, (T::ClassId, T::TokenId)), /* todo add tokenid and classid */
		Sent(T::AccountId, Vec<u8>),
		Transfer(T::AccountId, T::AccountId, Vec<u8>, (T::ClassId, T::TokenId)), /* todo add tokenid and classid */
		BindAddress(T::AccountId, Vec<u8>, AddressChainType, Vec<u8>),
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
				let next_id = orml_nft::Pallet::<T>::next_class_id(); // todo just use one class id
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
		DomainMustExist,
		UnSupportChainType,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn register(
			origin: OriginFor<T>,
			domain: Vec<u8>,
			bitcoin: Option<Vec<u8>>,
			ethereum: Option<Vec<u8>>,
			polkadot: Option<Vec<u8>>,
			kusama: Option<Vec<u8>>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			ensure!(
				domain.len() <= T::MaxDomainLen::get() as usize,
				Error::<T>::InvalidDomainLength
			);

			ensure!(!DomainInfos::<T>::contains_key(&domain), Error::<T>::DomainMustExist);

			let token_id = orml_nft::Pallet::<T>::next_token_id(T::NftClassID::get());
			let owner: T::AccountId = T::PalletId::get().into_sub_account(token_id);

			// todo, call nft::mint later
			let proxy_deposit = <pallet_proxy::Pallet<T>>::deposit(1u32);
			let total_deposit = proxy_deposit;

			// ensure enough token for proxy deposit + class deposit
			<T as pallet_proxy::Config>::Currency::transfer(
				&who,
				&owner,
				total_deposit,
				KeepAlive,
			)?;

			<pallet_proxy::Pallet<T>>::add_proxy_delegate(
				&owner,
				who.clone(),
				Default::default(),
				Zero::zero(),
			)?;

			let deposit = <T as nft::Config>::CreateTokenDeposit::get();
			let total_deposit = deposit.saturating_mul(1u32.into());
			<T as pallet_proxy::Config>::Currency::transfer(&who, &who, total_deposit, KeepAlive)?;
			<T as pallet_proxy::Config>::Currency::reserve(&who, total_deposit)?;
			let token_data = nft::TokenData { deposit };

			orml_nft::Pallet::<T>::mint(
				&who,
				T::NftClassID::get(),
				domain.clone(), // domain string
				token_data.clone(),
			)?;

			let polkadot_dest = match polkadot.clone() {
				Some(address) => Some(T::AccountId::decode(&mut &address[..]).unwrap_or_default()),
				None => None,
			};
			let kusama_dest = match kusama.clone() {
				Some(address) => Some(T::AccountId::decode(&mut &address[..]).unwrap_or_default()),
				None => None,
			};

			let deposit = T::DomainDeposit::get();
			<T as pallet::Config>::Currency::reserve(&who, deposit)?;
			<DomainInfos<T>>::insert(
				&domain,
				DomainInfo {
					native: who.clone(),
					bitcoin: bitcoin.clone(),
					ethereum: ethereum.clone(),
					polkadot: polkadot_dest,
					kusama: kusama_dest,
					deposit,
					nft_token: (T::NftClassID::get(), token_id.into()),
				},
			);
			<Domains<T>>::insert(&who, &domain);

			Self::deposit_event(Event::DomainRegistered(
				who,
				domain,
				bitcoin,
				ethereum,
				polkadot,
				kusama,
				deposit,
				(T::NftClassID::get(), token_id.into()),
			));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn deregister(origin: OriginFor<T>, domain: Vec<u8>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			let domain_info = <DomainInfos<T>>::take(&domain);
			nft::Pallet::<T>::burn(origin, domain_info.nft_token)?;

			<Domains<T>>::remove(&who);
			<T as pallet::Config>::Currency::unreserve(&who, domain_info.deposit);

			Self::deposit_event(Event::DomainDeregistered(who, domain, domain_info.nft_token));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn send(
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
		pub fn transfer(
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

				domain_info.native = to.clone();

				Self::deposit_event(Event::Transfer(
					who,
					to,
					domain.clone(),
					domain_info.nft_token,
				));

				Ok(().into())
			})?;

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn bind_address(
			origin: OriginFor<T>,
			domain: Vec<u8>,
			chain_type: AddressChainType,
			address: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			if Domains::<T>::contains_key(&who) {
				if domain == <Domains<T>>::get(&who) {
					DomainInfos::<T>::try_mutate(&domain, |domain_info| -> DispatchResult {
						match chain_type {
							AddressChainType::BTC => {
								domain_info.bitcoin = Some(address.clone());
							}
							AddressChainType::ETH => {
								domain_info.ethereum = Some(address.clone());
							}
							AddressChainType::DOT => {
								let new_address = address.clone(); //32 bytes
								let dest =
									T::AccountId::decode(&mut &new_address[..]).unwrap_or_default();

								domain_info.polkadot = Some(dest);
							}
							AddressChainType::KSM => {
								let new_address = address.clone(); //32 bytes
								let dest =
									T::AccountId::decode(&mut &new_address[..]).unwrap_or_default();

								domain_info.kusama = Some(dest);
							}
							_ => {
								Err(Error::<T>::UnSupportChainType)?;
							}
						}
						Self::deposit_event(Event::BindAddress(
							who,
							domain.clone(),
							chain_type,
							address.clone(),
						));

						Ok(())
					})?;
				}
			}

			Ok(().into())
		}
	}
}
