//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;
use sp_std::vec;

#[allow(unused)]
use crate::Pallet as Deitos;
use frame_benchmarking::v2::*;
use frame_support::{
    dispatch::DispatchResult, pallet_prelude::*, traits::tokens::fungible::Mutate,
};
use frame_system::RawOrigin;
use log;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_ip() {
        let total_storage: StorageSizeMB = 1000u64;

        let caller = whitelisted_caller();

        let balance: BalanceOf<T> = BalanceOf::<T>::from(1_000_000_000_u32);

        T::Currency::mint_into(&caller, balance);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), total_storage);
    }

    impl_benchmark_test_suite!(Deitos, crate::tests::new_test_ext(), crate::tests::Test);
}
