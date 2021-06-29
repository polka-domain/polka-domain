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

use cumulus_primitives_core::ParaId;
use parachain_runtime::{AccountId, AuraId, DomainRegistrarConfig, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<parachain_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Polka Domain Development",
		"polka_domain_dev",
		ChainType::Development,
		move || {
			genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![get_from_seed::<AuraId>("Alice")],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions { relay_chain: "rococo-dev".into(), para_id: id.into() },
	)
}

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Polka Domain Local Testnet",
		"polka_domain_local_testnet",
		ChainType::Local,
		move || {
			genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions { relay_chain: "rococo-local".into(), para_id: id.into() },
	)
}

pub fn genesis_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Polka Domain",
		"polka_domain_mainnet",
		ChainType::Local,
		move || {
			genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![get_from_seed::<AuraId>("Alice"), get_from_seed::<AuraId>("Bob")],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions { relay_chain: "rococo".into(), para_id: id.into() },
	)
}

fn genesis(
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> parachain_runtime::GenesisConfig {
	parachain_runtime::GenesisConfig {
		frame_system: parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: parachain_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		pallet_sudo: parachain_runtime::SudoConfig { key: root_key },
		parachain_info: parachain_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_aura: parachain_runtime::AuraConfig { authorities: initial_authorities },
		cumulus_pallet_aura_ext: Default::default(),
		orml_nft: Default::default(),
		orml_tokens: Default::default(),
		nft: Default::default(),
		domain_registrar: DomainRegistrarConfig {
			domains: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				br#"polkadot.domain"#.to_vec(),
				br#"0x0000"#.to_vec(),
			)],
		},
	}
}
