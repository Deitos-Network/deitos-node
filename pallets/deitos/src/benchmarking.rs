//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;
use sp_std::vec;

#[allow(unused)]
use crate::Pallet as Deitos;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn register_ip() {
        let value = 100u32.into();
        let total_storage = 1000u64.into();

        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        _(RawOrigin::Signed(caller), total_storage);
    }

    impl_benchmark_test_suite!(Deitos, crate::mock::new_test_ext(), crate::mock::Test);
}
