// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
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

//! Some configurable implementations as associated type for the substrate runtime.

use crate::{AccountId, Assets, Authorship, Balance, Balances, NegativeImbalance, Runtime};
use frame_support::traits::{
	fungibles::{Balanced, CreditOf},
	Currency, OnUnbalanced,
};
use pallet_asset_tx_payment::HandleCredit;
use sp_staking::{EraIndex, OnStakerSlash};
use sp_std::collections::btree_map::BTreeMap;
use tfchain_support::traits::FindNextAuthor;

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		if let Some(author) = Authorship::author() {
			Balances::resolve_creating(&author, amount);
		}
	}
}

/// A `HandleCredit` implementation that naively transfers the fees to the block author.
/// Will drop and burn the assets in case the transfer fails.
pub struct CreditToBlockAuthor;
impl HandleCredit<AccountId, Assets> for CreditToBlockAuthor {
	fn handle_credit(credit: CreditOf<AccountId, Assets>) {
		if let Some(author) = pallet_authorship::Pallet::<Runtime>::author() {
			// Drop the result which will trigger the `OnDrop` of the imbalance in case of error.
			let _ = Assets::resolve(&author, credit);
		}
	}
}

pub struct OnStakerSlashNoop;
impl OnStakerSlash<AccountId, Balance> for OnStakerSlashNoop {
	fn on_slash(
		_stash: &AccountId,
		_slashed_active: Balance,
		_slashed_ongoing: &BTreeMap<EraIndex, Balance>,
	) {
		// do nothing
	}
}

pub struct FindNextAuraAuthor;
impl FindNextAuthor<AccountId> for FindNextAuraAuthor {
	fn is_next_block_author(account: AccountId) -> Result<bool, ()> {
		let author = match <pallet_authorship::Pallet<Runtime>>::author() {
			Some(a) => a,
			None => return Ok(false),
		};
		let validators = <pallet_session::Pallet<Runtime>>::validators();

		let validator_count = validators.len();
		let author_index =
			(validators.iter().position(|a| a == &author).unwrap_or(0) + 1) % validator_count;

		return Ok(account == validators[author_index]);
	}

	fn is_validator(account: AccountId) -> bool {
		<pallet_session::Pallet<Runtime>>::validators().contains(&account)
	}
}
