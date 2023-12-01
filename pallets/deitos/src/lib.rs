// Deitos pallet
// Documentation under development !!!
#![allow(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]
#[warn(unused_imports)]
#[cfg(test)]
mod mock;

pub use pallet::*;

#[cfg(test)]
mod tests;

mod types;
pub use types::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        tokens::fungible::{
            self,
            hold::{
                Balanced as BalancedHold, Mutate as FunHoldMutate, Unbalanced as FunHoldUnbalanced,
            },
            Inspect as FunInspect,
        },
        Get,
    },
    PalletId,
};
pub use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_std::{convert::TryInto, prelude::*};

use sp_runtime::{
    traits::{One, Saturating, StaticLookup, Zero},
    BoundedVec,
};
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_system::pallet_prelude::*;
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The fungible used for deposits.
        type Currency: FunHoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
            + FunInspect<Self::AccountId>
            + BalancedHold<Self::AccountId>
            + FunHoldUnbalanced<Self::AccountId>;

        /// Overarching hold reason.
        type RuntimeHoldReason: From<HoldReason>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;

        type AgreementId: Member
            + Parameter
            + Copy
            + Clone
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + Saturating
            + One
            + Zero;

        /// Maximum Plan Duration
        #[pallet::constant]
        type MaxPaymentPlanDuration: Get<u32>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    /// A reason for the NIS pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        #[codec(index = 0)]
        Transfer,
    }

    #[pallet::storage]
    #[pallet::getter(fn get_ip)]
    pub type InfrastructureProvider<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, InfraProviderDetails<T>>;

    #[pallet::storage]
    #[pallet::getter(fn get_agreements)]
    pub type Agreements<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,   // consumer
            NMapKey<Blake2_128Concat, T::AccountId>,   // infrastructure provider
            NMapKey<Blake2_128Concat, T::AgreementId>, // agreement id
        ),
        AgreementDetails<T>,
        ResultQuery<Error<T>::NonExistentStorageValue>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has successfully set a new value.
        SomethingStored {
            /// The new value set.
            something: u32,
            /// The account who set the new value.
            who: T::AccountId,
        },
    }

    /// information.
    #[pallet::error]
    pub enum Error<T> {
        /// The value retrieved was `None` as no value was previously set.
        NoneValue,
        StorageOverflow,
        NonExistentStorageValue,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_ip())]
        /* to add more parameters*/
        pub fn register_ip(origin: OriginFor<T>, price_unit: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_ip_details())]
        pub fn update_ip_details(origin: OriginFor<T>, price_unit: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::shutdown_ip())]
        pub fn shutdown_ip(origin: OriginFor<T>, price_unit: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::shutdown_ip())]
        pub fn submit_agreement_request(
            origin: OriginFor<T>,
            provider: AccountIdLookupOf<T>,
            storage: Storage,
            time_allocation: AgreementTimeAllocation,
            activation_block: BlockNumberFor<T>,
            payment_plan: PaymentPlan<T>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::ip_agreement_reponse())]
        pub fn ip_agreement_reponse(
            origin: OriginFor<T>,
            consumer: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
            payment_plan: Option<PaymentPlan<T>>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Accepts the agreement and payment plan
            // Accepts the agreement and propose a payment plan with modifications
            // Rejects the agreement and payment plan

            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::consumer_agreement_reponse())]
        pub fn consumer_agreement_reponse(
            origin: OriginFor<T>,
            provider: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Accepts the agreement if IP accepts everything without modifications
            // Accepts the payment plan with modifications
            // Rejects the plan with modifications

            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::consumer_cancels_agreement())]
        pub fn consumer_cancels_agreement(
            origin: OriginFor<T>,
            provider: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::ip_cancels_agreement())]
        pub fn ip_cancels_agreement(
            origin: OriginFor<T>,
            consumer: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::make_installment_payment())]
        pub fn make_installment_payment(
            origin: OriginFor<T>,
            provider: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::withdraw_provider_funds())]
        pub fn withdraw_provider_funds(
            origin: OriginFor<T>,
            consumer: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::submit_provider_feedback())]
        pub fn submit_provider_feedback(
            origin: OriginFor<T>,
            consumer: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::submit_consumer_feedback())]
        pub fn submit_consumer_feedback(
            origin: OriginFor<T>,
            provider: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }
    }
}
