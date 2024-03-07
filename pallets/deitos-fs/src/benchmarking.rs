//! Benchmarking setup for pallet-deitos.

// Copyright (C) NC2D Labs.
// This file is part of Deitos Node.

// Deitos Node is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Deitos Node is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Deitos Node.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]
#![allow(warnings)]

use frame_benchmarking::v2::*;
use frame_support::{pallet_prelude::*, traits::tokens::fungible::Mutate};
use frame_system::RawOrigin;
use log;

#[allow(unused)]
use crate::Pallet as Deitos;

use super::*;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn ip_register() {
        let total_storage: StorageSizeMB = 1000u64;

        let caller = whitelisted_caller();

        let balance: BalanceOf<T> = BalanceOf::<T>::from(1_000_000_000_u32);

        T::Currency::mint_into(&caller, balance);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), total_storage);
    }

    impl_benchmark_test_suite!(Deitos, crate::tests::new_test_ext(), crate::tests::Test);
}
