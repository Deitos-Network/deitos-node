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
use scale_info::prelude::string::String;
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

#[allow(missing_docs)]
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

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;

        /// The fungible used for deposits.
        type Currency: FunHoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
            + FunInspect<Self::AccountId>
            + FunMutate<Self::AccountId>
            + BalancedHold<Self::AccountId>
            + FunHoldUnbalanced<Self::AccountId>;

        /// Overarching hold reason.
        type RuntimeHoldReason: From<HoldReason>;

        /// Agreement Id type
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

        /// Payment Plan Limit
        #[pallet::constant]
        type PaymentPlanLimit: Get<u32>;

        /// Agreements per IP Limit
        #[pallet::constant]
        type IPAgreementsLimit: Get<u32>;

        /// Agreements per Consumer Limit
        #[pallet::constant]
        type ConsumerAgreementsLimit: Get<u32>;

        /// Pallet ID
        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    /// A reason for the Deitos pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// Initial deposit for IP registration
        IPInitialDeposit,
        /// Consumer service deposit
        ConsumerServiceDeposit,
        /// Consumer deposit for securing an agreement
        ConsumerSecurityDeposit,
        /// Consumer installment payment
        ConsumerInstallment,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// The amount of initial deposit for IP registration
        pub ip_initial_deposit: BalanceOf<T>,
        /// The amount of service deposit for consumer
        pub consumer_service_deposit: BalanceOf<T>,
        /// The price for storage of 1 MB per block
        pub price_storage_mb_per_block: BalanceOf<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            IPDepositAmount::<T>::put(self.ip_initial_deposit);
            ConsumerServiceDepositAmount::<T>::put(self.consumer_service_deposit);
            CurrentPrices::<T>::put(Prices {
                storage_mb_per_block: self.price_storage_mb_per_block,
            });
        }
    }

    /// The amount of initial deposit for IP registration
    #[pallet::storage]
    #[pallet::getter(fn ip_deposit_amount)]
    pub type IPDepositAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The amount of service deposit for consumer
    #[pallet::storage]
    #[pallet::getter(fn consumer_service_deposit_amount)]
    pub type ConsumerServiceDepositAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Prices defined by the protocol
    #[pallet::storage]
    #[pallet::getter(fn ip_cost_per_unit)]
    pub type CurrentPrices<T: Config> = StorageValue<_, Prices<T>, ValueQuery>;

    /// IPs currently existing in the network
    #[pallet::storage]
    #[pallet::getter(fn get_ip)]
    pub type InfrastructureProviders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, IPDetails<T>>;

    /// Agreements currently existing in the network
    #[pallet::storage]
    #[pallet::getter(fn get_agreement)]
    pub(super) type Agreements<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AgreementId, AgreementDetails<T>>;

    /// Customers` agreements currently existing in the network. This is a mapping from the customer
    /// to a vector of agreement ids.
    #[pallet::storage]
    #[pallet::getter(fn get_consumer_agreement)]
    pub(super) type ConsumerAgreements<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ConsumerAgreementsVec<T>, ValueQuery>;

    /// Current agreement id. This is used to assign a unique id to each created agreement.
    /// The id is incremented by one for each new agreement.
    #[pallet::storage]
    #[pallet::getter(fn current_agreement_id)]
    pub type CurrentAgreementId<T: Config> = StorageValue<_, T::AgreementId, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// An IP has been registered
        IPRegistered {
            /// The IP that has been registered
            ip: T::AccountId,
            /// The total storage of the IP
            total_storage: StorageSizeMB,
        },
        /// An IP has updated its storage amount
        IPStorageUpdated {
            /// The IP that has updated its storage amount
            ip: T::AccountId,
            /// The new total storage of the IP
            total_storage: StorageSizeMB,
        },
        /// An IP has updated its status
        IPStatusChanged {
            /// The IP that has updated its status
            ip: T::AccountId,
            /// The new status of the IP
            status: IPStatus,
        },
        /// An IP has been unregistered
        IPUnregistered {
            /// The IP that has been unregistered
            ip: T::AccountId,
        },
        /// The price for storage per block has been updated
        StoragePriceUnitUpdated {
            /// The new price for storage per block
            price_storage_per_block: BalanceOf<T>,
        },
        /// An agreement status has changed
        AgreementStatusChanged {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The new status of the agreement
            status: AgreementStatus,
        },
        /// A consumer has requested an agreement
        ConsumerRequestedAgreement {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The IP the agreement is with
            ip: T::AccountId,
            /// The consumer requesting the agreement
            consumer: T::AccountId,
            /// The total deposit the consumer has payed
            consumer_total_deposit: BalanceOf<T>,
            /// The amount of storage covered by the agreement
            storage: StorageSizeMB,
            /// The block number when the rental starts
            activation_block: BlockNumberFor<T>,
            /// The payment plan for the agreement
            payment_plan: PaymentPlan<T>,
        },
        /// A consumer has revoked an agreement
        ConsumerRevokedAgreement {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The IP the agreement is with
            ip: T::AccountId,
            /// The consumer revoking the agreement
            consumer: T::AccountId,
            /// The total deposit released
            consumer_total_deposit: BalanceOf<T>,
        },
        /// An IP has accepted an agreement
        IPAcceptedAgreement {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The IP accepting the agreement
            ip: T::AccountId,
            /// The consumer the agreement is with
            consumer: T::AccountId,
        },
        /// An IP has proposed a new payment plan
        IPProposedPaymentPlan {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The IP proposing the new payment plan
            ip: T::AccountId,
            /// The consumer the agreement is with
            consumer: T::AccountId,
            /// The new payment plan
            payment_plan: PaymentPlan<T>,
        },
        /// A consumer has accepted an agreement
        ConsumerAcceptedAgreement {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The IP the agreement is with
            ip: T::AccountId,
            /// The consumer accepting the agreement
            consumer: T::AccountId,
            /// The previously held security deposit, which is released now
            consumer_security_deposit_released: BalanceOf<T>,
            /// The new security deposit, which is held now
            consumer_security_deposit_held: BalanceOf<T>,
        },
        /// A consumer has prepaid an installment
        ConsumerPrepaidInstallment {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The consumer prepaying the installment
            consumer: T::AccountId,
            /// The cost of the installment
            cost: BalanceOf<T>,
        },
        /// An IP has withdrawn installments
        IPWithdrewInstallments {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The IP withdrawing the installments
            ip: T::AccountId,
            /// The total amount withdrawn
            transferred: BalanceOf<T>,
        },
        /// An IP has terminated an agreement due to non-payment
        IPTerminatedNonPay {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The IP terminating the agreement
            ip: T::AccountId,
            /// The total amount transferred to the IP
            transferred: BalanceOf<T>,
        },
        /// A consumer has submitted feedback
        ConsumerSubmittedFeedback {
            /// The agreement id
            agreement_id: T::AgreementId,
            /// The consumer submitting the feedback
            consumer: T::AccountId,
            /// The IP the agreement is with
            ip: T::AccountId,
            /// The performance score
            score_performance: Score,
            /// The stability score
            score_stability: Score,
            /// The support score
            score_support: Score,
            /// The comment of the feedback
            comment: String,
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
        /// No unpaid installments found.
        /// E.g., the consumer cannot prepay any more installments, or the IP cannot terminate the agreement
        /// due to non-payment, because all installments have been paid.
        NoUnpaidInstallments,
        /// Agreement not found for consumer
        NoAgreementForConsumer,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register an IP. The IP must not be registered already, or must have been unregistered.
        /// The IP must pay a deposit to register. The deposit is returned when the IP unregisters.
        /// The IP must also specify the total storage it has.
        /// The IP is registered with status `Pending` and must be activated by the network operator.
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

            let ip_details = IPDetails::new(total_storage, Self::ip_deposit_amount());
            InfrastructureProviders::<T>::insert(&ip, ip_details);

            Self::deposit_event(Event::IPRegistered { ip, total_storage });
            Ok(())
        }

        /// Update the status of an IP. Only the network operator can update the status of an IP.
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

        /// Update the total storage of an IP.
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

        /// Unregister an IP. The IP must be registered and must not have any agreements in progress.
        /// The IP gets back the deposit it payed during registration.
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

        /// Update the price for storage per block. Only the network operator can update the price.
        /// This change doesn't affect installments that have already been paid.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_storage_cost_per_unit())]
        pub fn update_storage_cost_per_unit(
            origin: OriginFor<T>,
            price_storage_per_block: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            CurrentPrices::<T>::put(Prices {
                storage_mb_per_block: price_storage_per_block,
            });

            Self::success_event(Event::StoragePriceUnitUpdated {
                price_storage_per_block,
            })
        }

        /// Request an agreement with an IP. The IP must be registered and active. The consumer must
        /// pay a deposit to secure the agreement. The deposit is returned if the consumer revokes
        /// the agreement, or is used to pay for the last installment. The consumer must specify the
        /// amount of storage it needs, the block number when the rental starts and the payment plan.
        ///
        /// The payment plan must is a vector of block numbers. Every element represents the
        /// end of an installment. The first installment starts at the activation block. The last element
        /// is the end of the rental. The payment plan must be strictly increasing and contain at least 1 element.
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

            let consumer_total_deposit =
                agreement.hold_consumer_deposits(Self::consumer_service_deposit_amount())?;

            let agreement_id = Self::insert_agreement(agreement)?;

            Self::success_event(Event::ConsumerRequestedAgreement {
                agreement_id,
                ip,
                consumer,
                consumer_total_deposit,
                storage,
                activation_block,
                payment_plan,
            })
        }

        /// Revoke an agreement. The agreement must be in progress. The consumer gets back the deposit
        /// it payed to secure the agreement. Only the consumer can revoke the agreement. The agreement
        /// can be revoked only if it is not accepted yet.
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

            let consumer_total_deposit = agreement.release_consumer_deposits()?;

            Self::delete_agreement(agreement_id)?;

            Self::success_event(Event::ConsumerRevokedAgreement {
                agreement_id,
                ip: agreement.ip,
                consumer,
                consumer_total_deposit,
            })
        }

        /// Accept an agreement by the IP that the agreement is with. The activation block must not be
        /// in the past. The status of the agreement must be `ConsumerRequest`. The agreement is
        /// accepted by the IP and the status changes to `Active`.
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

                    agreement.update_status(agreement_id, AgreementStatus::Active);
                    Ok(agreement.consumer.clone())
                },
            )?;

            Self::success_event(Event::IPAcceptedAgreement {
                agreement_id,
                ip,
                consumer,
            })
        }

        /// Propose a new payment plan for an agreement. The agreement status must be `ConsumerRequest`.
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

                    agreement.payment_plan = payment_plan.clone();
                    agreement.update_status(agreement_id, AgreementStatus::IPProposedPaymentPlan);

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

        /// Accept a payment plan proposed by an IP. The agreement status must be `IPProposedPaymentPlan`.
        /// The consumer deposit is adjusted to the new payment plan. The agreement status changes to `Active`.
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::consumer_accept_agreement())]
        pub fn consumer_accept_agreement(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let (ip, consumer_security_deposit_released, consumer_security_deposit_held) =
                Agreements::<T>::try_mutate(
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

                        let old_deposit = agreement.consumer_security_deposit;
                        let new_deposit = agreement.adjust_consumer_security_deposit()?;

                        agreement.update_status(agreement_id, AgreementStatus::Active);
                        Ok((agreement.ip.clone(), old_deposit, new_deposit))
                    },
                )?;

            Self::success_event(Event::ConsumerAcceptedAgreement {
                agreement_id,
                ip,
                consumer,
                consumer_security_deposit_released,
                consumer_security_deposit_held,
            })
        }

        /// Prepay an installment. The agreement status must be `Active`. The consumer pays the cost
        /// of the next unpaid installment. All payments are saved in the agreement's payment history.
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::consumer_prepay_installment())]
        pub fn consumer_prepay_installment(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let cost = Agreements::<T>::try_mutate(
                agreement_id,
                |agreement| -> Result<_, DispatchError> {
                    let agreement = agreement.as_mut().ok_or(Error::<T>::AgreementNotFound)?;

                    // Check that the transaction was signed by the consumer
                    ensure!(
                        agreement.consumer == consumer,
                        Error::<T>::AgreementNotFound
                    );

                    // Check that the agreement is in progress
                    ensure!(
                        agreement.status == AgreementStatus::Active,
                        Error::<T>::AgreementStatusInvalid
                    );

                    let cost = agreement.hold_next_installment()?;
                    Ok(cost)
                },
            )?;

            Self::success_event(Event::ConsumerPrepaidInstallment {
                agreement_id,
                consumer,
                cost,
            })
        }

        /// Withdraw installments. The agreement status must be `Active`. The IP withdraws all complete installments
        /// from the agreement. The IP can withdraw installments only if the consumer has prepaid them.
        ///
        /// If the agreement is fully paid, the status changes to `Completed`.
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::ip_withdraw_installments())]
        pub fn ip_withdraw_installments(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            let ip = ensure_signed(origin)?;

            let transferred = Agreements::<T>::try_mutate(
                agreement_id,
                |agreement| -> Result<_, DispatchError> {
                    let agreement = agreement.as_mut().ok_or(Error::<T>::AgreementNotFound)?;

                    // Check that the transaction was signed by the IP
                    ensure!(agreement.ip == ip, Error::<T>::AgreementNotFound);

                    // Check that the agreement is in progress
                    ensure!(
                        agreement.status == AgreementStatus::Active,
                        Error::<T>::AgreementStatusInvalid
                    );

                    let current_block_number = Self::current_block_number();
                    let transferred = agreement.transfer_installments(current_block_number)?;

                    // Check if all installments have been withdrawn
                    if agreement.consumer_security_deposit_transferred {
                        agreement.update_status(agreement_id, AgreementStatus::Completed);
                    }

                    Ok(transferred)
                },
            )?;

            Self::success_event(Event::IPWithdrewInstallments {
                agreement_id,
                ip,
                transferred,
            })
        }

        /// Terminate an agreement due to non-payment. The agreement status must be `Active`. The IP
        /// receives all unpaid installments and the consumer deposit. The agreement is deleted.
        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::ip_terminate_nonpay())]
        pub fn ip_terminate_nonpay(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
        ) -> DispatchResult {
            let ip = ensure_signed(origin)?;

            let mut agreement =
                Self::get_agreement(agreement_id).ok_or(Error::<T>::AgreementNotFound)?;

            // Check that the transaction was signed by the IP
            ensure!(agreement.ip == ip, Error::<T>::AgreementNotFound);

            // Check that the agreement is in progress
            ensure!(
                agreement.status == AgreementStatus::Active,
                Error::<T>::AgreementStatusInvalid
            );

            let current_block_number = Self::current_block_number();
            let transferred = agreement
                .has_overdue_installments(current_block_number)
                .then(|| -> Result<_, DispatchError> {
                    let installments = agreement.transfer_installments(current_block_number)?;
                    let security_deposit = agreement.transfer_consumer_security_deposit()?;
                    let service_deposit = agreement.transfer_consumer_service_deposit()?;
                    Self::delete_agreement(agreement_id)?;

                    Ok(installments
                        .saturating_add(security_deposit)
                        .saturating_add(service_deposit))
                })
                .ok_or(Error::<T>::NoUnpaidInstallments)??;

            Self::success_event(Event::IPTerminatedNonPay {
                agreement_id,
                ip,
                transferred,
            })
        }

        /// Submit feedback for an agreement. The agreement status must be `Completed`. The consumer
        /// submits a score and a comment. The completed agreement is deleted. The consumer service
        /// deposit is released.
        #[pallet::call_index(13)]
        #[pallet::weight(T::WeightInfo::consumer_submit_feedback())]
        pub fn consumer_submit_feedback(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
            score_performance: Score,
            score_stability: Score,
            score_support: Score,
            comment: String,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let mut agreement =
                Agreements::<T>::get(agreement_id).ok_or(Error::<T>::AgreementNotFound)?;

            // Check that the transaction was signed by the consumer
            ensure!(
                agreement.consumer == consumer,
                Error::<T>::AgreementNotFound
            );

            // Check that the agreement is completed
            ensure!(
                agreement.status == AgreementStatus::Completed,
                Error::<T>::AgreementStatusInvalid
            );

            //Save the score. The IP must exist.
            InfrastructureProviders::<T>::mutate(&agreement.ip, |ip_details| {
                ip_details
                    .as_mut()
                    .map(|x| x.add_score(score_performance, score_stability, score_support))
            });

            agreement.release_consumer_deposits()?;

            Self::delete_agreement(agreement_id)?;

            Self::success_event(Event::ConsumerSubmittedFeedback {
                agreement_id,
                consumer,
                ip: agreement.ip,
                score_performance,
                score_stability,
                score_support,
                comment,
            })
        }
    }
}

// TODO: Move this to a utils module
fn is_strictly_increasing<T: PartialOrd>(slice: &[T]) -> bool {
    slice.windows(2).all(|window| window[0] < window[1])
}
