
//! Autogenerated weights for pallet_template
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-04-06, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Alexs-MacBook-Pro-2.local`, CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ../../target/release/node-template
// benchmark
// pallet
// --chain
// dev
// --pallet
// pallet_template
// --extrinsic
// *
// --steps=50
// --repeat=20
// --wasm-execution=compiled
// --output
// pallets/template/src/weights.rs
// --template
// ../../.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for pallet_template.
pub trait WeightInfo {
	fn register_ip() -> Weight;
	fn update_ip_status() -> Weight;
	fn update_ip_storage() -> Weight;
	fn unregister_ip() -> Weight;
	fn update_storage_cost_per_unit() -> Weight;
	fn submit_agreement_request() -> Weight;
	fn ip_agreement_reponse() -> Weight;
	fn consumer_agreement_reponse() -> Weight;
	fn consumer_cancels_agreement() -> Weight;
	fn ip_cancels_agreement() -> Weight;
	fn make_installment_payment() -> Weight;
	fn withdraw_provider_funds() -> Weight;
	fn submit_provider_feedback() -> Weight;	
	fn submit_consumer_feedback() -> Weight;

}

/// Weights for pallet_template using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn register_ip() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn update_ip_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn update_ip_storage() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	
	fn unregister_ip() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn update_storage_cost_per_unit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn submit_agreement_request() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn ip_agreement_reponse() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn consumer_agreement_reponse() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn consumer_cancels_agreement() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn ip_cancels_agreement() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn make_installment_payment() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn withdraw_provider_funds() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn submit_provider_feedback() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}

	fn submit_consumer_feedback() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn register_ip() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn update_ip_status() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn update_ip_storage() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
		.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn unregister_ip() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}

	fn update_storage_cost_per_unit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	


		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn submit_agreement_request() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn ip_agreement_reponse() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn consumer_agreement_reponse() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn consumer_cancels_agreement() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn ip_cancels_agreement() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn make_installment_payment() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn withdraw_provider_funds() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn submit_provider_feedback() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
		/// Storage: Deitos Something (r:0 w:1)
	/// Proof: Deitos Something (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	fn submit_consumer_feedback() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 8_000_000 picoseconds.
		Weight::from_parts(10_000_000, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
}
