// This file is part of Polka Domain.

// Copyright (C) 2020-2021 Polka Domain Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use sha3::{Digest, Keccak256};
use std::convert::TryInto;
use syn::{parse_macro_input, Expr, ExprLit, Ident, ItemEnum, Lit};

#[proc_macro_attribute]
pub fn generate_function_selector(_: TokenStream, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemEnum);

	let ItemEnum {
		attrs,
		vis,
		enum_token,
		ident,
		variants,
		..
	} = item;

	let mut ident_expressions: Vec<Ident> = vec![];
	let mut variant_expressions: Vec<Expr> = vec![];
	for variant in variants {
		if let Some((_, Expr::Lit(ExprLit { lit, .. }))) = variant.discriminant {
			if let Lit::Str(token) = lit {
				let selector = get_function_selector(&token.value());
				// println!("method: {:?}, selector: {:?}", token.value(), selector);
				ident_expressions.push(variant.ident);
				variant_expressions.push(Expr::Lit(ExprLit {
					lit: Lit::Verbatim(Literal::u32_suffixed(selector)),
					attrs: Default::default(),
				}));
			} else {
				panic!("Not method string: `{:?}`", lit);
			}
		} else {
			panic!("Not enum: `{:?}`", variant);
		}
	}

	(quote! {
		#(#attrs)*
		#vis #enum_token #ident {
			#(
				#ident_expressions = #variant_expressions,
			)*
		}
	})
	.into()
}

fn get_function_selector(s: &str) -> u32 {
	// create a SHA3-256 object
	let mut hasher = Keccak256::new();
	// write input message
	hasher.update(s);
	// read hash digest
	let result = hasher.finalize();
	u32::from_be_bytes(result[..4].try_into().unwrap())
}