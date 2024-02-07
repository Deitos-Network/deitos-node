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

#![deny(missing_docs)]
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

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_file())]
        pub fn register_file(origin: OriginFor<T>, agreement_id: T::AgreementId, md5: [u8; 32] ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            pallet_deitos::Pallet::<T>::consumer_has_agreement(&consumer,&agreement_id)?;

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
