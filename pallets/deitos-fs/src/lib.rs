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
        ConstU32, Get,
    },
    PalletId,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
pub use sp_runtime::{
    offchain::{
        http,
        storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
        Duration,
    },
    traits::{One, Saturating, StaticLookup, TrailingZeroInput, Zero},
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    BoundedVec, RuntimeDebug, SaturatedConversion,
};

use frame_support::traits::Randomness;
use frame_system::offchain::{
    AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
    SignedPayload, Signer, SigningTypes, SubmitTransaction, SendTransactionTypes
};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};
use scale_info::prelude::format;
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
    pub trait Config:
		SendTransactionTypes<Call<Self>> + frame_system::Config + pallet_deitos::Config
    {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;

        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

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
            + Zero
            + Into<u32>
            + From<u32>;

        /// Pallet ID
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::storage]
    /// The holdings of a specific account for a specific asset.
    pub(super) type Files<T: Config> = StorageMap<_, Blake2_128Concat, T::FileId, FileDetails<T>>;

    #[pallet::storage]
    pub type CurrentFileId<T: Config> = StorageValue<_, T::FileId, ValueQuery>;

    #[pallet::storage]
    /// The holdings of a specific account for a specific asset.
    pub(super) type FilesToBeChecked<T: Config> =
        StorageMap<_, Blake2_128Concat, T::FileId, FileDetails<T>>;

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
            md5: [u8; 64],
        },
        FileVerified {
            /// The file id
            file_id: T::FileId,
        },
        FileConflict {
            /// The file id
            file_id: T::FileId,
        },
        FileNotVerified {
            /// The file id
            file_id: T::FileId,
            /// Error count
            error_count: u32,
        },
        DataIntegrityCheckSuccessful {
            /// The file id
            file_id: T::FileId,
        },
        DataIntegrityCheckFailed {
            /// The file id
            file_id: T::FileId,
        },
    }

    /// Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// IP agreements limit reached
        IPAgreementsLimit,
        OffchainUnsignedTxError,
    }

    /// Hook
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            for (file_id, file) in FilesToBeChecked::<T>::iter() {
            
                let name = sp_std::str::from_utf8(file.file_name.as_slice()).unwrap();
                let hadoop_file_hash = Self::fetch_file_hash(&name).unwrap();

                Self::offchain_unsigned_tx(file_id, file, hadoop_file_hash);
            }
            return;

         /*   let last_file_id = CurrentFileId::<T>::get();
            if last_file_id.is_zero() {
                return;
            }
            let last_file_id: u32 = last_file_id.saturated_into();

            let phrase = b"deitos-fs-offchain-worker";
            let (seed, block) = T::Randomness::random(phrase);

            let seed_as_bytes = seed.encode();
            let encoded_block_number = block_number.encode();
            let combined_seed =
                [seed_as_bytes.as_slice(), encoded_block_number.as_slice()].concat();
            let hash_of_combined_seed = sp_io::hashing::blake2_256(&combined_seed);
            let seed_array: [u8; 32] = {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&hash_of_combined_seed);
                arr
            };
            let mut rng = ChaChaRng::from_seed(seed_array);
            let random_value = rng.next_u32();
            let file_id: T::FileId = (1 + (random_value % last_file_id)).into();
            let file: FileDetails<T> = Files::<T>::get(file_id).unwrap();
            let name = sp_std::str::from_utf8(file.file_name.as_slice()).unwrap();
            let hadoop_file_hash = Self::fetch_file_hash(name).unwrap();
            if file.md5 == hadoop_file_hash {
                Self::deposit_event(Event::DataIntegrityCheckSuccessful { file_id });
            } else {
                Self::deposit_event(Event::DataIntegrityCheckFailed { file_id });
            } */
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;

        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let valid_tx = |provide| {
                ValidTransaction::with_tag_prefix("ocw-demo")
                    .priority(100_u64)
                    .and_provides([&provide])
                    .longevity(3)
                    .propagate(true)
                    .build()
            };

            match call {
                Call::submit_file_validation {
                    file_id: _file_id,
                    file: file_id,
                    returned_hash: _returned_hash,
                } => valid_tx(b"submit_file_validation".to_vec()),
                _ => InvalidTransaction::Call.into(),
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_file())]
        pub fn register_file(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
            md5: [u8; 64],
            file_name: FileName,
        ) -> DispatchResult {
            let _consumer = ensure_signed(origin)?;

            // TODO commented for quick testing
            //  pallet_deitos::Pallet::<T>::consumer_has_agreement(&consumer,&agreement_id)?;

            let file_id: T::FileId = Self::next_file_id();

            let file = FileDetails::<T>::new(agreement_id, md5, file_name);

            FilesToBeChecked::<T>::insert(file_id, file);

            Self::deposit_event(Event::FileRegistered {
                agreement_id,
                file_id,
                md5,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::register_file())]
        pub fn submit_file_validation(
            origin: OriginFor<T>,
            file_id: T::FileId,
            file: FileDetails<T>,
            returned_hash: [u8; 64],
        ) -> DispatchResult {
            ensure_none(origin)?;
            let mut new_file = file.clone();
            if file.md5 == returned_hash && file.status == FileValidationStatus::Pending {
                new_file.status = FileValidationStatus::Verified;
                Files::<T>::insert(file_id, new_file);
                FilesToBeChecked::<T>::remove(file_id);
                Self::deposit_event(Event::FileVerified { file_id });
                log::info!("ALL GOOD");
            } else {
                FilesToBeChecked::<T>::mutate(file_id, |file_option| {
                    if let Some(file_check) = file_option {
                        file_check.error_count = file_check.error_count.saturating_add(1);

                        if file_check.error_count > 3 {
                            file_check.status = FileValidationStatus::Conflict;
                            log::info!("INCREASE COUNT");
                            Self::deposit_event(Event::FileConflict { file_id });
                        } else {
                            log::info!("FILE NOT VERIFIED");
                            Self::deposit_event(Event::FileNotVerified {
                                file_id,
                                error_count: file_check.error_count,
                            });
                        }
                    }
                });
            }
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn fetch_file_hash(file_id: &str) -> Result<[u8; 64], http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
        let url = format!("http://localhost:3030/calculate/{}", file_id);
        let request = http::Request::get(&url);
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
        // Let's check the status code before we proceed to reading the response.
        if response.code != 200 {
            log::warn!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        let body = response.body().collect::<Vec<u8>>();
        let mut array = [0u8; 64];
        let bytes_to_copy = body.len().min(64);
        array[..bytes_to_copy].copy_from_slice(&body[..bytes_to_copy]);

        Ok(array)
    }

    fn offchain_unsigned_tx(
        file_id: T::FileId,
        file: FileDetails<T>,
        returned_hash: [u8; 64],
    ) -> Result<(), Error<T>> {
        let call = Call::submit_file_validation {
            file_id,
            file,
            returned_hash,
        };

        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(|_| {
            log::error!("Failed in offchain_unsigned_tx");
            <Error<T>>::OffchainUnsignedTxError
        })
    }
}
