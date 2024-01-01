//! Benchmarking setup for pallet-deitos.

// TODO: Implement benchmarks for the pallet.

#![cfg(feature = "runtime-benchmarks")]
// TODO: Remove this
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
