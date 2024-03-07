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
//! The File System Deitos pallet provides functionality for related to file management and integrity checks.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        tokens::fungible::{
            hold::{Balanced as BalancedHold, Unbalanced as FunHoldUnbalanced},
            Inspect as FunInspect, Mutate as FunMutate,
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
        Duration, Timestamp,
    },
    traits::{One, Saturating, StaticLookup, TrailingZeroInput, Zero},
    transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
    BoundedVec, RuntimeDebug, SaturatedConversion,
};

use frame_support::traits::Randomness;
use frame_system::offchain::{SendTransactionTypes, SubmitTransaction};
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

        type Randomness: Randomness<Option<Self::Hash>, BlockNumberFor<Self>>;

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

        #[pallet::constant]
        type Seed: Get<u32>;

        #[pallet::constant]
        type ErrorBoundary: Get<u32>;
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
            /// File hash
            hash: FileHash,
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
        /// Off chain unsigned Tx Error.
        OffchainUnsignedTxError,
        /// Some internal error during the check
        CheckDataInternalFailure,
        /// File fetched Failed
        FileFetchFailed,
    }

    /// Hook
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            for (file_id, file) in FilesToBeChecked::<T>::iter() {
                let name = sp_std::str::from_utf8(file.file_name.as_slice()).unwrap();
                let hadoop_file_hash = Self::fetch_file_hash(&name).unwrap();

                let _ = Self::unsigned_file_upload(file_id, file, hadoop_file_hash)
                    .map_err(|_| <Error<T>>::FileFetchFailed);
            }

            let seed: u32 = T::Seed::get();
            if Self::is_current_block_eligible(block_number.saturated_into(), seed) {
                let current_file_id = CurrentFileId::<T>::get();
                if current_file_id.is_zero() {
                    return;
                }
                let (file_id, result) = Self::check_data_integrity_protocol().unwrap();
                let _ = Self::unsigned_check_integrity(file_id, result)
                    .map_err(|_| <Error<T>>::FileFetchFailed);
            }
        }
    }

    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            let valid_tx = |provide| {
                ValidTransaction::with_tag_prefix("ocw-deitos")
                    .priority(100_u64)
                    .and_provides([&provide])
                    .longevity(3)
                    .propagate(true)
                    .build()
            };

            match call {
                Call::submit_file_validation {
                    file_id: _file_id,
                    file: _file,
                    returned_hash: _returned_hash,
                } => valid_tx(b"submit_file_validation".to_vec()),
                Call::data_integrity_protocol {
                    file_id: _file_id,
                    result: _result,
                } => valid_tx(b"data_integrity_protocol".to_vec()),
                _ => InvalidTransaction::Call.into(),
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// This call register a file for a certain agreement. It checks the consumer has an active agreement that allows the upload.
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_file())]
        pub fn register_file(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
            hash: FileHash,
            file_name: FileName,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;
            pallet_deitos::Pallet::<T>::consumer_has_agreement(&consumer, &agreement_id)?;

            let file_id: T::FileId = Self::next_file_id();

            let file = FileDetails::<T>::new(agreement_id, hash, file_name);

            FilesToBeChecked::<T>::insert(file_id, file);

            Self::deposit_event(Event::FileRegistered {
                agreement_id,
                file_id,
                hash,
            });
            Ok(())
        }

        /// Unsigned call to submit the file validation from the offchain worker.
        /// It checks the hash returned from the offchain worker and updates the file status if it matches.
        /// In case not, it increases the error count.
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_file_validation())]
        pub fn submit_file_validation(
            origin: OriginFor<T>,
            file_id: T::FileId,
            file: FileDetails<T>,
            returned_hash: FileHash,
        ) -> DispatchResult {
            ensure_none(origin)?;
            let mut new_file = file.clone();
            if file.hash == returned_hash && file.status == FileValidationStatus::Pending {
                new_file.status = FileValidationStatus::Verified;

                Files::<T>::insert(file_id, new_file);
                FilesToBeChecked::<T>::remove(file_id);
                Self::deposit_event(Event::FileVerified { file_id });
            } else {
                FilesToBeChecked::<T>::mutate(file_id, |file_option| {
                    if let Some(file_check) = file_option {
                        let error_boundary: u32 = T::ErrorBoundary::get();
                        file_check.error_count = file_check.error_count.saturating_add(1);

                        if file_check.error_count > error_boundary {
                            file_check.status = FileValidationStatus::Conflict;
                            Self::deposit_event(Event::FileConflict { file_id });
                        } else {
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

        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::data_integrity_protocol())]
        pub fn data_integrity_protocol(
            origin: OriginFor<T>,
            file_id: T::FileId,
            result: CheckResult,
        ) -> DispatchResult {
            ensure_none(origin)?;
            match result {
                CheckResult::CheckPassed => {
                    Self::deposit_event(Event::DataIntegrityCheckSuccessful { file_id });
                }
                CheckResult::DataIntegrityCheckFailed => {
                    Self::deposit_event(Event::DataIntegrityCheckFailed { file_id });
                }
            }

            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn fetch_file_hash(file_id: &str) -> Result<FileHash, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
        let url = format!("http://verifier.deitos.network:4040/{}", file_id);
        let request = http::Request::get(&url);
        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;
        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;
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

    fn unsigned_file_upload(
        file_id: T::FileId,
        file: FileDetails<T>,
        returned_hash: FileHash,
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

    fn unsigned_check_integrity(file_id: T::FileId, result: CheckResult) -> Result<(), Error<T>> {
        let call = Call::data_integrity_protocol { file_id, result };

        SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into()).map_err(|_| {
            log::error!("Failed in offchain_unsigned_tx");
            <Error<T>>::OffchainUnsignedTxError
        })
    }

    fn check_data_integrity_protocol() -> Result<(T::FileId, CheckResult), Error<T>> {
        let last_file_id = CurrentFileId::<T>::get();
        log::info!("DI last_file_id {:?}", last_file_id);
        let last_file_id: u32 = last_file_id.saturated_into();

        let phrase = b"deitos-fs-offchain-worker";
        let (seed, _block) = T::Randomness::random(phrase);
        let seed_as_bytes = seed.encode(); // This gives you a Vec<u8>
        let seed_slice = seed_as_bytes.as_slice(); // Convert Vec<u8> to &[u8]
        let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed_slice))
            .map_err(|_| <Error<T>>::CheckDataInternalFailure)?;

        let mut rng = ChaChaRng::from_seed(seed);
        let random_value = rng.next_u32();
        let file_id: T::FileId = (1 + (random_value % last_file_id)).into();
        let file: FileDetails<T> =
            Files::<T>::get(file_id).ok_or(Error::<T>::CheckDataInternalFailure)?;
        let name = sp_std::str::from_utf8(file.file_name.as_slice())
            .map_err(|_| <Error<T>>::CheckDataInternalFailure)?;
        let hadoop_file_hash =
            Self::fetch_file_hash(name).map_err(|_| <Error<T>>::FileFetchFailed)?;
        if file.hash == hadoop_file_hash {
            Ok((file_id, CheckResult::CheckPassed))
        } else {
            Ok((file_id, CheckResult::DataIntegrityCheckFailed))
        }
    }

    fn is_current_block_eligible(current_block_number: u32, seed: u32) -> bool {
        let range_size = 10;
        let range_index = (current_block_number - 1) / range_size;
        let offset = (seed.wrapping_add(range_index as u32) ^ seed) % range_size;
        let eligible_block_number = range_index * range_size + offset + 1;
        current_block_number == eligible_block_number
    }
}
