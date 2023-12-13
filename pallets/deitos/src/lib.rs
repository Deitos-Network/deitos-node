// Deitos pallet
// Documentation under development !!!
#![allow(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]
#[warn(unused_imports)]
#[cfg(test)]
mod mock;

pub use pallet::*;
mod impls;
pub use impls::*;
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
        tokens::{
            fungible::{
                self,
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
pub use log;
pub use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_std::{convert::TryInto, prelude::*};

use sp_runtime::{
    traits::{CheckedMul, One, Saturating, StaticLookup, Zero},
    BoundedVec, SaturatedConversion,
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
            + FunMutate<Self::AccountId>
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

        /// Maximum Plan Duration
        #[pallet::constant]
        type MaxAgreements: Get<u32>;

        #[pallet::constant]
        type MaxAgreementsPerConsumer: Get<u32>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    /// A reason for the NIS pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        #[codec(index = 0)]
        IPInitialDeposit,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Genesis Initial IP Deposit
        pub initial_ip_deposit: BalanceOf<T>,
        pub initial_ip_costs_per_unit: BalanceOf<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            IPDepositAmount::<T>::put(&self.initial_ip_deposit);
            IPUnitCosts::<T>::put(IPCostsPerUnit {
                price_storage_per_block: self.initial_ip_costs_per_unit,
            });
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn ip_deposit_amount)]
    pub type IPDepositAmount<T: Config> =
        StorageValue<_, BalanceOf<T>, ResultQuery<Error<T>::NonExistentStorageValue>>;

    #[pallet::storage]
    #[pallet::getter(fn ip_cost_per_unit)]
    pub type IPUnitCosts<T: Config> =
        StorageValue<_, IPCostsPerUnit<T>, ResultQuery<Error<T>::NonExistentStorageValue>>;

    #[pallet::storage]
    #[pallet::getter(fn get_ip)]
    pub type InfrastructureProvider<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, IPDetails<T>>;

    #[pallet::storage]
    #[pallet::getter(fn agreements)]
    pub(super) type Agreements<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AgreementId,
        AgreementDetails<T>,
        ResultQuery<Error<T>::NonExistentStorageValue>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn consumer_agreement)]
    pub(super) type ConsumerAgreements<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId, // Consumer
        AgreementsPerConsumer<T>,
        ResultQuery<Error<T>::NonExistentStorageValue>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn last_id)]
    pub type LastId<T: Config> = StorageValue<_, T::AgreementId, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A user has successfully set a new value.
        IPRegistered {
            ip: T::AccountId,
            total_storage: StorageSizeMB,
        },
        IPStorageUpdated {
            ip: T::AccountId,
            total_storage: StorageSizeMB,
        },
        IPStatusChanged {
            ip: T::AccountId,
            status: IPStatus,
        },
        IPUnregistered {
            ip: T::AccountId,
        },
        StoragePriceUnitUpdated {
            price_storage_per_block: BalanceOf<T>,
        },
        ConsumerRequestedAgreement {
            ip: T::AccountId,
            consumer: T::AccountId,
            status: AgreementStatus,
            storage: StorageSizeMB,
            activation_block: BlockNumberFor<T>,
            payment_plan: PaymentPlan<T>,
        },
    }

    /// information.
    #[pallet::error]
    pub enum Error<T> {
        /// The value retrieved was `None` as no value was previously set.
        NonExistentStorageValue,
        /// Math overflow
        Overflow,
        /// Insufficient storage
        InsufficientStorage,
        /// On going agreements
        OnGoingAgreements,
        /// IP already exists,
        IPAlreadyExists,
        /// IP not active
        IPNotActive,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_ip())]
        pub fn register_ip(origin: OriginFor<T>, total_storage: StorageSizeMB) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let ip = ensure_signed(origin)?;

            // Checks that the IP is either not registered or is registered but with Unregistered status
            if let Some(ip_details) = InfrastructureProvider::<T>::get(&ip) {
                ensure!(
                    ip_details.status == IPStatus::Unregistered,
                    Error::<T>::IPAlreadyExists
                );
            }

            let deposit_amount = IPDepositAmount::<T>::get()?;

            T::Currency::hold(&HoldReason::IPInitialDeposit.into(), &ip, deposit_amount)?;

            let ip_details = IPDetails::<T> {
                total_storage,
                reserved_storage: Zero::zero(),
                status: IPStatus::Validating,
                active_agreements: BoundedVec::new(),
                deposit_amount,
            };

            InfrastructureProvider::<T>::insert(&ip, ip_details);
            Self::deposit_event(Event::IPRegistered { ip, total_storage });

            Ok(())
        }

        /// This is a temporary call to manage the IP status.
        /// Statuses updates should be done automatically after an environment software check.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_ip_status())]
        pub fn update_ip_status(
            origin: OriginFor<T>,
            ip: AccountIdLookupOf<T>,
            status: IPStatus,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let provider = T::Lookup::lookup(ip)?;

            InfrastructureProvider::<T>::try_mutate(
                &provider,
                |ip_details| -> Result<_, DispatchError> {
                    let ip_details = ip_details
                        .as_mut()
                        .ok_or(Error::<T>::NonExistentStorageValue)?;

                    ip_details.status = status.clone();

                    Ok(())
                },
            )?;

            Self::deposit_event(Event::IPStatusChanged {
                ip: provider.clone(),
                status,
            });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_ip_storage())]
        pub fn update_ip_storage(
            origin: OriginFor<T>,
            total_storage: StorageSizeMB,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let ip = ensure_signed(origin)?;

            InfrastructureProvider::<T>::try_mutate(
                &ip,
                |ip_details| -> Result<_, DispatchError> {
                    let ip_details = ip_details
                        .as_mut()
                        .ok_or(Error::<T>::NonExistentStorageValue)?;

                    // Check if the new total_storage is enough to cover the current reserved_storage
                    ensure!(
                        total_storage >= ip_details.reserved_storage,
                        Error::<T>::InsufficientStorage
                    );

                    ip_details.total_storage = total_storage;

                    Ok(())
                },
            )?;

            Self::deposit_event(Event::IPStorageUpdated {
                ip: ip.clone(),
                total_storage,
            });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::unregister_ip())]
        pub fn unregister_ip(origin: OriginFor<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let ip = ensure_signed(origin)?;

            InfrastructureProvider::<T>::try_mutate(
                &ip,
                |ip_details| -> Result<_, DispatchError> {
                    let ip_details = ip_details
                        .as_mut()
                        .ok_or(Error::<T>::NonExistentStorageValue)?;

                    ensure!(
                        ip_details.active_agreements.len() == 0,
                        Error::<T>::OnGoingAgreements
                    );

                    ip_details.status = IPStatus::Unregistered;

                    T::Currency::release(
                        &HoldReason::IPInitialDeposit.into(),
                        &ip,
                        ip_details.deposit_amount,
                        Exact,
                    )?;

                    Ok(())
                },
            )?;

            Self::deposit_event(Event::IPUnregistered { ip: ip.clone() });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_storage_cost_per_unit())]
        pub fn update_storage_cost_per_unit(
            origin: OriginFor<T>,
            price_storage_per_block: BalanceOf<T>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            ensure_root(origin)?;

            IPUnitCosts::<T>::put(IPCostsPerUnit {
                price_storage_per_block: price_storage_per_block.clone(),
            });

            Self::deposit_event(Event::StoragePriceUnitUpdated {
                price_storage_per_block,
            });

            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::unregister_ip())]
        pub fn submit_agreement_request(
            origin: OriginFor<T>,
            ip: AccountIdLookupOf<T>,
            storage: StorageSizeMB,
            activation_block: BlockNumberFor<T>,
            proposed_plan: ProposedPlan<T>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let consumer = ensure_signed(origin)?;
            let ip = T::Lookup::lookup(ip)?;
            // Checks that the IP is either not registered or is registered but with Unregistered status
            if let Some(ip_details) = InfrastructureProvider::<T>::get(&ip) {
                ensure!(
                    ip_details.status == IPStatus::Active,
                    Error::<T>::IPNotActive
                );
                ensure!(
                    ip_details
                        .total_storage
                        .saturating_sub(ip_details.reserved_storage)
                        >= storage,
                    Error::<T>::InsufficientStorage
                );
            }

            let agreement_id = Self::next_agreement_id()?;

            let cost_per_block = IPUnitCosts::<T>::get()?;
            let price_storage_per_block = cost_per_block.price_storage_per_block;

            let proposed_plan_with_balances: Vec<PaymentsDetails<T>> = proposed_plan
                .iter()
                .map(|(k, v)| {
                    let blocks_amount = v.saturating_sub(*k).saturated_into::<u128>();
                    let storage_price = price_storage_per_block.saturated_into::<u128>();

                    let price = storage_price.saturating_mul(blocks_amount);
                    let p: BalanceOf<T> = BalanceOf::<T>::saturated_from(price);

                    (k.clone(), v.clone(), p)
                })
                .collect();

            let payment_plan: PaymentPlan<T> =
                PaymentPlan::<T>::try_from(proposed_plan_with_balances)
                    .map_err(|_| Error::<T>::Overflow)?;

            let agreement = AgreementDetails::<T> {
                ip: ip.clone(),
                consumer: consumer.clone(),
                status: AgreementStatus::ConsumerRequest,
                storage,
                activation_block,
                payment_plan: payment_plan.clone(),
            };

            Agreements::<T>::insert(&agreement_id, agreement);
            Self::deposit_event(Event::ConsumerRequestedAgreement {
                ip,
                consumer,
                status: AgreementStatus::ConsumerRequest,
                storage,
                activation_block,
                payment_plan,
            });

            Ok(())
        }

        #[pallet::call_index(6)]
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

        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::consumer_agreement_reponse())]
        pub fn consumer_agreement_reponse(
            origin: OriginFor<T>,
            ip: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            // Accepts the agreement if IP accepts everything without modifications
            // Accepts the payment plan with modifications
            // Rejects the plan with modifications

            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::consumer_cancels_agreement())]
        pub fn consumer_cancels_agreement(
            origin: OriginFor<T>,
            ip: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(9)]
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

        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::make_installment_payment())]
        pub fn make_installment_payment(
            origin: OriginFor<T>,
            ip: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(11)]
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

        #[pallet::call_index(12)]
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

        #[pallet::weight(T::WeightInfo::submit_consumer_feedback())]
        pub fn submit_consumer_feedback(
            origin: OriginFor<T>,
            ip: AccountIdLookupOf<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let who = ensure_signed(origin)?;

            Ok(())
        }
    }
}
