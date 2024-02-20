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

//! # Deitos Pallet
//!
//! The Deitos pallet implements the Deitos protocol. It allows Infrastructure Providers (IPs) to
//! register and manage their storage capacity and consumers to request storage capacity from IPs.
//! The protocol is designed to be flexible and allow for different payment plans.


#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        tokens::{
            fungible::{
                hold::{
                    Balanced as BalancedHold, Mutate as FunHoldMutate,
                    Unbalanced as FunHoldUnbalanced,
                },
                Inspect as FunInspect, Mutate as FunMutate,
            },
            Precision::Exact,
        },
        Get,
    },
    PalletId,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::{
	offchain::{
		http,
		storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
		Duration,
	},
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	RuntimeDebug,
    traits::{One, Saturating, StaticLookup, Zero},
    SaturatedConversion,
};

use sp_std::{convert::TryInto, prelude::*};


#[warn(unused_imports)]
pub use pallet::*;
pub use types::*;
pub use weights::*;

mod impls;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod types;

#[allow(missing_docs)]
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;

    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_deitos::Config  {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;

        /// The fungible used for deposits.
        type Currency: FunInspect<Self::AccountId>
            + FunMutate<Self::AccountId>
            + BalancedHold<Self::AccountId>
            + FunHoldUnbalanced<Self::AccountId>;

        /// Agreement Id type
        type FileId: Member
            + Default
            + Parameter
            + Copy
            + Clone
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + Saturating
            + One
            + Zero;


        /// Pallet ID
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }


    #[pallet::storage]
    /// The holdings of a specific account for a specific asset.
    pub(super) type Files<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AgreementId,
        Blake2_128Concat,
        T::FileId,
        FileDetails
    >;

    #[pallet::storage]
    #[pallet::getter(fn current_agreement_id)]
    pub type CurrentFileId<T: Config> = StorageValue<_, T::FileId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// File was registered
        FileRegistered {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The file id
            file_id: T::FileId,
            /// File MD5
            md5: [u8; 32]
        }
    }

    /// Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// IP agreements limit reached
        IPAgreementsLimit,
    }

/*     /// Hook
    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_block_number: BlockNumberFor<T>) {
			log::info!("Hello World from offchain workers!");

       //   let res = Self::fetch_file_hash(1,"test.csv").unwrap();
         // log::info!("value: {:?}",res);
     
		}
	}
 */

/* 	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;

		/// Validate unsigned call to this module.
		///
		/// By default unsigned transactions are disallowed, but implementing the validator
		/// here we make sure that some particular calls (the ones produced by offchain worker)
		/// are being whitelisted and marked as valid.
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			// Firstly let's check that we call the right function.
			if let Call::submit_price_unsigned_with_signed_payload {
				price_payload: ref payload,
				ref signature,
			} = call
			{
				let signature_valid =
					SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
				if !signature_valid {
					return InvalidTransaction::BadProof.into()
				}
				Self::validate_transaction_parameters(&payload.block_number, &payload.price)
			} else if let Call::submit_price_unsigned { block_number, price: new_price } = call {
				Self::validate_transaction_parameters(block_number, new_price)
			} else {
				InvalidTransaction::Call.into()
			}
		}
	} */

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_file())]
        pub fn register_file(origin: OriginFor<T>, agreement_id: T::AgreementId, md5: [u8; 32] ) -> DispatchResult {
            let _consumer = ensure_signed(origin)?;

          //  pallet_deitos::Pallet::<T>::consumer_has_agreement(&consumer,&agreement_id)?;

            let file_id: T::FileId = Self::next_file_id();

            let file = FileDetails::new(md5);

            Files::<T>::insert(agreement_id,file_id,file);

            Self::deposit_event(Event::FileRegistered { agreement_id, file_id, md5 });
            Ok(())
        }
    }
}

// TODO: Move this to a utils module
fn is_strictly_increasing<T: PartialOrd>(slice: &[T]) -> bool {
    slice.windows(2).all(|window| window[0] < window[1])
}

/// Test
pub trait ValidateUnsigned {
	/// The call to validate
	type Call;

	/// Validate the call right before dispatch.
	///
	/// This method should be used to prevent transactions already in the pool
	/// (i.e. passing [`validate_unsigned`](Self::validate_unsigned)) from being included in blocks
	/// in case they became invalid since being added to the pool.
	///
	/// By default it's a good idea to call [`validate_unsigned`](Self::validate_unsigned) from
	/// within this function again to make sure we never include an invalid transaction. Otherwise
	/// the implementation of the call or this method will need to provide proper validation to
	/// ensure that the transaction is valid.
	///
	/// Changes made to storage *WILL* be persisted if the call returns `Ok`.
	fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
		Self::validate_unsigned(TransactionSource::InBlock, call)
			.map(|_| ())
			.map_err(Into::into)
	}

	/// Return the validity of the call
	///
	/// This method has no side-effects. It merely checks whether the call would be rejected
	/// by the runtime in an unsigned extrinsic.
	///
	/// The validity checks should be as lightweight as possible because every node will execute
	/// this code before the unsigned extrinsic enters the transaction pool and also periodically
	/// afterwards to ensure the validity. To prevent dos-ing a network with unsigned
	/// extrinsics, these validity checks should include some checks around uniqueness, for example,
	/// checking that the unsigned extrinsic was sent by an authority in the active set.
	///
	/// Changes made to storage should be discarded by caller.
	fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity;
}

impl<T: Config> Pallet<T> {
    	/// Fetch current price and return the result in cents.
	fn fetch_file_hash(randon_value: u32, file_id: &str) -> Result<String, http::Error> {
		// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
		// deadline to 2s to complete the external call.
		// You can also wait indefinitely for the response, however you may still get a timeout
		// coming from the host machine.
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
		let url =  format!("http://localhost:3030/calculate/{}/{}", randon_value, file_id);
		let request =
			http::Request::get(&url);
		// We set the deadline for sending of the request, note that awaiting response can
		// have a separate deadline. Next we send the request, before that it's also possible
		// to alter request headers or stream body content in case of non-GET requests.
		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

		// The request is already being processed by the host, we are free to do anything
		// else in the worker (we can send multiple concurrent requests too).
		// At some point however we probably want to check the response though,
		// so we can block current thread and wait for it to finish.
		// Note that since the request is being driven by the host, we don't have to wait
		// for the request to have it complete, we will just not read the response.
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
		// Let's check the status code before we proceed to reading the response.
		if response.code != 200 {
			log::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown)
		}

		// Next we want to fully read the response body and collect it to a vector of bytes.
		// Note that the return object allows you to read the body in chunks as well
		// with a way to control the deadline.
		let body = response.body().collect::<Vec<u8>>();

		// Create a str slice from the body.
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;

		log::info!("Got hash: {}", body_str);

		Ok(body_str.to_string())
	}

}