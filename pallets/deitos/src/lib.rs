// Deitos pallet
// TODO: Add documentation
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
pub use log;
pub use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::{
    traits::{One, Saturating, StaticLookup, Zero},
    BoundedVec, SaturatedConversion,
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
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;

    use super::*;

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
            + Default
            + Parameter
            + Copy
            + Clone
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + Saturating
            + One
            + Zero;

        /// Maximum Payment Plan Limit
        #[pallet::constant]
        type PaymentPlanLimit: Get<u32>;

        /// Maximum Agreements
        #[pallet::constant]
        type IPAgreementsLimit: Get<u32>;

        #[pallet::constant]
        type ConsumerAgreementsLimit: Get<u32>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    /// A reason for the NIS pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        #[codec(index = 0)]
        IPInitialDeposit,
        #[codec(index = 1)]
        ConsumerDeposit,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Genesis Initial IP Deposit
        pub initial_ip_deposit: BalanceOf<T>,
        pub initial_price_storage_mb_per_block: BalanceOf<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            IPDepositAmount::<T>::put(&self.initial_ip_deposit);
            CurrentPrices::<T>::put(Prices {
                storage_mb_per_block: self.initial_price_storage_mb_per_block,
            });
        }
    }

    #[pallet::storage]
    #[pallet::getter(fn ip_deposit_amount)]
    pub type IPDepositAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Prices defined by the protocol
    #[pallet::storage]
    #[pallet::getter(fn ip_cost_per_unit)]
    pub type CurrentPrices<T: Config> = StorageValue<_, Prices<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_ip)]
    pub type InfrastructureProviders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, IPDetails<T>>;

    #[pallet::storage]
    #[pallet::getter(fn get_agreement)]
    pub(super) type Agreements<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AgreementId, AgreementDetails<T>>;

    #[pallet::storage]
    #[pallet::getter(fn get_consumer_agreement)]
    pub(super) type ConsumerAgreements<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ConsumerAgreementsVec<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn current_agreement_id)]
    pub type CurrentAgreementId<T: Config> = StorageValue<_, T::AgreementId, ValueQuery>;

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
            agreement_id: T::AgreementId,
            ip: T::AccountId,
            consumer: T::AccountId,
            consumer_deposit: BalanceOf<T>,
            storage: StorageSizeMB,
            activation_block: BlockNumberFor<T>,
            payment_plan: PaymentPlan<T>,
        },
        ConsumerRevokedAgreement {
            agreement_id: T::AgreementId,
            ip: T::AccountId,
            consumer: T::AccountId,
        },
        IPAcceptedAgreement {
            agreement_id: T::AgreementId,
            ip: T::AccountId,
            consumer: T::AccountId,
        },
        IPProposedPaymentPlan {
            agreement_id: T::AgreementId,
            ip: T::AccountId,
            consumer: T::AccountId,
            payment_plan: PaymentPlan<T>,
        },
        ConsumerAcceptedAgreement {
            agreement_id: T::AgreementId,
            ip: T::AccountId,
            consumer: T::AccountId,
        },
    }

    /// Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// IP agreements limit reached
        IPAgreementsLimit,
        /// Consumer agreements limit reached
        ConsumerAgreementsLimit,
        /// Insufficient storage
        InsufficientStorage,
        /// Payment plan invalid
        PaymentPlanInvalid,
        /// IP already exists,
        IPAlreadyExists,
        /// IP not found
        IPNotFound,
        /// IP not active
        IPNotActive,
        /// Agreement not found
        AgreementNotFound,
        /// Activation block invalid
        AgreementOutdated,
        /// On going agreement(s)
        AgreementInProgress,
        /// Agreement status invalid
        AgreementStatusInvalid,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::ip_register())]
        pub fn ip_register(origin: OriginFor<T>, total_storage: StorageSizeMB) -> DispatchResult {
            let ip = ensure_signed(origin)?;

            // Checks that the IP is either not registered or is registered but with Unregistered status
            if let Some(ip_details) = Self::get_ip(&ip) {
                ensure!(
                    ip_details.status == IPStatus::Unregistered,
                    Error::<T>::IPAlreadyExists
                );
            }

            T::Currency::hold(
                &HoldReason::IPInitialDeposit.into(),
                &ip,
                Self::ip_deposit_amount(),
            )?;

            let ip_details = IPDetails::<T> {
                total_storage,
                status: IPStatus::Pending,
                agreements: BoundedVec::new(),
                deposit: Self::ip_deposit_amount(),
            };

            InfrastructureProviders::<T>::insert(&ip, ip_details);

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
            let ip = T::Lookup::lookup(ip)?;

            InfrastructureProviders::<T>::try_mutate(&ip, |ip_details| {
                ip_details
                    .as_mut()
                    .map(|x| x.status = status)
                    .ok_or(Error::<T>::IPNotFound)
            })?;

            Self::success_event(Event::IPStatusChanged { ip, status })
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::ip_update_storage())]
        pub fn ip_update_storage(
            origin: OriginFor<T>,
            total_storage: StorageSizeMB,
        ) -> DispatchResult {
            let ip = ensure_signed(origin)?;

            InfrastructureProviders::<T>::try_mutate(&ip, |ip_details| {
                ip_details
                    .as_mut()
                    .map(|x| x.total_storage = total_storage)
                    .ok_or(Error::<T>::IPNotFound)
            })?;

            Self::success_event(Event::IPStorageUpdated { ip, total_storage })
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::ip_unregister())]
        pub fn ip_unregister(origin: OriginFor<T>) -> DispatchResult {
            let ip = ensure_signed(origin)?;

            InfrastructureProviders::<T>::try_mutate(
                &ip,
                |ip_details| -> Result<_, DispatchError> {
                    let ip_details = ip_details.as_mut().ok_or(Error::<T>::IPNotFound)?;

                    // TODO: Agreements at negotiation stage should not prevent the IP from unregistering.
                    ensure!(
                        ip_details.agreements.len() == 0,
                        Error::<T>::AgreementInProgress
                    );

                    ip_details.status = IPStatus::Unregistered;

                    T::Currency::release(
                        &HoldReason::IPInitialDeposit.into(),
                        &ip,
                        ip_details.deposit,
                        Exact,
                    )?;

                    Ok(())
                },
            )?;

            Self::success_event(Event::IPUnregistered { ip })
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_storage_cost_per_unit())]
        pub fn update_storage_cost_per_unit(
            origin: OriginFor<T>,
            price_storage_per_block: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            CurrentPrices::<T>::put(Prices {
                storage_mb_per_block: price_storage_per_block.clone(),
            });

            Self::success_event(Event::StoragePriceUnitUpdated {
                price_storage_per_block,
            })
        }

        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::consumer_request_agreement())]
        #[frame_support::transactional]
        pub fn consumer_request_agreement(
            origin: OriginFor<T>,
            ip: AccountIdLookupOf<T>,
            storage: StorageSizeMB,
            activation_block: BlockNumberFor<T>,
            payment_plan: PaymentPlan<T>,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;
            // Activation block must be in the future
            ensure!(
                activation_block > Self::current_block_number(),
                Error::<T>::AgreementOutdated
            );

            // Payment plan must be valid
            ensure!(
                Self::is_valid_payment_plan(&payment_plan, activation_block),
                Error::<T>::PaymentPlanInvalid
            );

            let ip = T::Lookup::lookup(ip)?;
            let ip_details =
                InfrastructureProviders::<T>::get(&ip).ok_or(Error::<T>::IPNotFound)?;

            // IP is active
            ensure!(
                ip_details.status == IPStatus::Active,
                Error::<T>::IPNotActive
            );

            // IP has enough storage
            ensure!(
                storage > Zero::zero() && storage <= ip_details.total_storage,
                Error::<T>::InsufficientStorage
            );

            let mut agreement = AgreementDetails::new_consumer_request(
                ip.clone(),
                consumer.clone(),
                storage,
                activation_block,
                payment_plan.clone(),
            );

            let consumer_deposit = agreement.hold_consumer_deposit()?;

            let agreement_id = Self::create_agreement(agreement)?;

            Self::success_event(Event::ConsumerRequestedAgreement {
                agreement_id,
                ip,
                consumer,
                consumer_deposit,
                storage,
                activation_block,
                payment_plan,
            })
        }

        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::consumer_revoke_agreement())]
        pub fn consumer_revoke_agreement(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let mut agreement =
                Self::get_agreement(agreement_id).ok_or(Error::<T>::AgreementNotFound)?;

            // Check that the transaction was signed by the consumer
            ensure!(
                agreement.consumer == consumer,
                Error::<T>::AgreementNotFound
            );

            // Check that the agreement is not in progress
            ensure!(
                agreement.status == AgreementStatus::ConsumerRequest
                    || agreement.status == AgreementStatus::IPProposedPaymentPlan,
                Error::<T>::AgreementInProgress
            );

            agreement.release_consumer_deposit()?;

            Self::delete_agreement(agreement_id)?;

            Self::success_event(Event::ConsumerRevokedAgreement {
                agreement_id,
                ip: agreement.ip,
                consumer,
            })
        }

        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::ip_accept_agreement())]
        pub fn ip_accept_agreement(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            let ip = ensure_signed(origin)?;

            let consumer = Agreements::<T>::try_mutate(
                agreement_id,
                |agreement| -> Result<_, DispatchError> {
                    let agreement = agreement.as_mut().ok_or(Error::<T>::AgreementNotFound)?;

                    // Check that the transaction was signed by the IP
                    ensure!(agreement.ip == ip, Error::<T>::AgreementNotFound);

                    // Check that the agreement is requested by the consumer
                    ensure!(
                        agreement.status == AgreementStatus::ConsumerRequest,
                        Error::<T>::AgreementStatusInvalid
                    );

                    // Activation block must not be in the past. Otherwise an IP can accept an old agreement
                    // and penalize the consumer for not paying.
                    ensure!(
                        agreement.activation_block >= Self::current_block_number(),
                        Error::<T>::AgreementOutdated
                    );

                    agreement.status = AgreementStatus::Agreed;
                    Ok(agreement.consumer.clone())
                },
            )?;

            Self::success_event(Event::IPAcceptedAgreement {
                agreement_id,
                ip,
                consumer,
            })
        }

        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::ip_propose_payment_plan())]
        pub fn ip_propose_payment_plan(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
            payment_plan: PaymentPlan<T>,
        ) -> DispatchResult {
            let ip = ensure_signed(origin)?;

            let consumer = Agreements::<T>::try_mutate(
                agreement_id,
                |agreement| -> Result<_, DispatchError> {
                    let agreement = agreement.as_mut().ok_or(Error::<T>::AgreementNotFound)?;

                    // Check that the transaction was signed by the IP
                    ensure!(agreement.ip == ip, Error::<T>::AgreementNotFound);

                    // Check that the agreement is requested by the consumer
                    ensure!(
                        agreement.status == AgreementStatus::ConsumerRequest,
                        Error::<T>::AgreementStatusInvalid
                    );

                    // Payment plan must be valid
                    ensure!(
                        Self::is_valid_payment_plan(&payment_plan, agreement.activation_block),
                        Error::<T>::PaymentPlanInvalid
                    );

                    agreement.status = AgreementStatus::IPProposedPaymentPlan;
                    agreement.payment_plan = payment_plan.clone();

                    Ok(agreement.consumer.clone())
                },
            )?;

            Self::success_event(Event::IPProposedPaymentPlan {
                agreement_id,
                ip,
                consumer,
                payment_plan,
            })
        }

        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::consumer_accept_agreement())]
        pub fn consumer_accept_agreement(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let ip = Agreements::<T>::try_mutate(
                agreement_id,
                |agreement| -> Result<_, DispatchError> {
                    let agreement = agreement.as_mut().ok_or(Error::<T>::AgreementNotFound)?;

                    // Check that the transaction was signed by the consumer
                    ensure!(
                        agreement.consumer == consumer,
                        Error::<T>::AgreementNotFound
                    );

                    // Check that IP has proposed a payment plan
                    ensure!(
                        agreement.status == AgreementStatus::IPProposedPaymentPlan,
                        Error::<T>::AgreementStatusInvalid
                    );

                    agreement.adjust_consumer_deposit()?;
                    agreement.status = AgreementStatus::Agreed;
                    Ok(agreement.ip.clone())
                },
            )?;

            Self::success_event(Event::ConsumerAcceptedAgreement {
                agreement_id,
                ip,
                consumer,
            })
        }

        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::make_installment_payment())]
        pub fn make_installment_payment(
            origin: OriginFor<T>,
            _ip: AccountIdLookupOf<T>,
            _agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let _who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::withdraw_provider_funds())]
        pub fn withdraw_provider_funds(
            origin: OriginFor<T>,
            _consumer: AccountIdLookupOf<T>,
            _agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let _who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::submit_provider_feedback())]
        pub fn submit_provider_feedback(
            origin: OriginFor<T>,
            _consumer: AccountIdLookupOf<T>,
            _agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let _who = ensure_signed(origin)?;

            Ok(())
        }

        #[pallet::call_index(13)]
        #[pallet::weight(T::WeightInfo::submit_consumer_feedback())]
        pub fn submit_consumer_feedback(
            origin: OriginFor<T>,
            _ip: AccountIdLookupOf<T>,
            _agreement_id: T::AgreementId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            let _who = ensure_signed(origin)?;

            Ok(())
        }
    }
}

// TODO: Move this to a utils module
fn is_strictly_increasing<T: PartialOrd>(slice: &[T]) -> bool {
    slice.windows(2).all(|window| window[0] < window[1])
}
