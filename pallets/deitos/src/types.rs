use core::cmp::Ordering;

use frame_support::traits::tokens::{Fortitude::Polite, Restriction::Free};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;

use crate::*;

/// Type alias for the balance type from the runtime.
pub type BalanceOf<T> =
    <<T as Config>::Currency as FunInspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Type alias for `AccountId` from the runtime.
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// Type alias for `AccountId` lookup from the runtime.
pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

/// Size of the storage in MB.
pub type StorageSizeMB = u64;

/// Payment plan for the agreement. The payment plan is a vector of block numbers. The first
/// element is the block number when the first installment is due. The last element is the block
/// number when the agreement ends. The difference between two consecutive elements is the length
/// of the installment in blocks.
pub type PaymentPlan<T> = BoundedVec<BlockNumberFor<T>, <T as Config>::PaymentPlanLimit>;

/// The vector of all the agreements for a single IP. The vector is bounded by the maximum number
/// of agreements per IP (IPAgreementsLimit).
pub type IPAgreementsVec<T> =
    BoundedVec<<T as Config>::AgreementId, <T as Config>::IPAgreementsLimit>;

/// The vector of all the agreements for a single consumer. The vector is bounded by the maximum
/// number of agreements per consumer (ConsumerAgreementsLimit).
pub type ConsumerAgreementsVec<T> =
    BoundedVec<<T as Config>::AgreementId, <T as Config>::ConsumerAgreementsLimit>;

/// The statuses an IP can have. When an IP is registered it has the status `Pending`. Then the IP
/// can be activated by the network operator and the status changes to `Active`. The IP can deactivate itself
/// and the status changes to `Unregistered`.
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub enum IPStatus {
    /// IP is registered but not activated yet
    Pending,
    /// IP is activated
    Active,
    /// IP is deactivated
    Unregistered,
}

/// The statuses an agreement can have. When a consumer requests an agreement the status is
/// `ConsumerRequest`. The IP can agree to the agreement and the status changes to `Active`, or
/// the IP can propose a payment plan and the status changes to `IPProposedPaymentPlan`. If the
/// consumer accepts the payment plan the status changes to `Active`.
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub enum AgreementStatus {
    /// Consumer requested an agreement
    ConsumerRequest,
    /// IP proposed a different payment plan
    IPProposedPaymentPlan,
    /// Agreement is active
    Active,
    /// Agreement is completed, meaning that the IP has received all the payments
    Completed,
}

/// The details of an IP. The IP has:
/// - `total_storage` - the total storage the IP has
/// - `status` - the current status of the IP
/// - `agreements` - the vector of all the agreements for this IP
/// - `deposit` - the deposit the IP has payed during the registration process
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct IPDetails<T: pallet::Config> {
    /// Total IP storage
    pub total_storage: StorageSizeMB,
    /// IP Status
    pub status: IPStatus,
    /// Track of active agreements
    pub agreements: IPAgreementsVec<T>,
    /// Deposit funds
    pub deposit: BalanceOf<T>,
}

/// An item of the payment history.
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct PaymentRecord<T: pallet::Config> {
    /// The amount of the installment. This is held from the consumer and transferred to the IP
    /// when the installment is due.
    pub amount: BalanceOf<T>,
    /// Flag indicating if the installment is transferred to the IP.
    pub transferred: bool,
}

/// A vector of payment records. The vector is bounded by the maximum number of installments
/// in the payment plan (PaymentPlanLimit).
pub type PaymentRecords<T> = BoundedVec<PaymentRecord<T>, <T as Config>::PaymentPlanLimit>;

/// The payment history for the agreement. The payment history has:
/// - `records` - the vector of payment records
/// - `next_transfer_installment_index` - the index of the next installment to be transferred to
/// the IP
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct PaymentHistory<T: pallet::Config> {
    /// Payment records
    pub records: PaymentRecords<T>,
    /// Next installment to be transferred to the IP
    pub next_transfer_installment_index: u32,
}

impl<T: pallet::Config> PaymentHistory<T> {
    fn new() -> Self {
        Self {
            records: PaymentRecords::new(),
            next_transfer_installment_index: 0,
        }
    }
}

/// The details of an agreement. The agreement has:
/// - `ip` - the IP the agreement is with
/// - `consumer` - the consumer the agreement is with
/// - `consumer_security_deposit` - the deposit the consumer has payed to secure the agreement
/// - `consumer_security_deposit_transferred` - flag indicating if the consumer security deposit is transferred to the IP
/// - `status` - the current status of the agreement
/// - `storage` - the amount of storage covered by the agreement
/// - `activation_block` - the block number when the rental starts
/// - `payment_plan` - the payment plan for the agreement
/// - `payment_history` - the payment history for the agreement
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct AgreementDetails<T: pallet::Config> {
    /// IP participating in the agreement
    pub ip: AccountIdOf<T>,
    /// Consumer participating in the agreement
    pub consumer: AccountIdOf<T>,
    /// Security deposit amount currently held from the consumer
    pub consumer_security_deposit: BalanceOf<T>,
    /// Flag indicating if the consumer security deposit is transferred to the IP
    pub consumer_security_deposit_transferred: bool,
    /// Current status of the agreement
    pub status: AgreementStatus,
    /// The amount of storage covered by the agreement
    pub storage: StorageSizeMB,
    /// The block number when the rental starts
    pub activation_block: BlockNumberFor<T>,
    /// The payment plan for the agreement
    pub payment_plan: PaymentPlan<T>,
    /// The payment history for the agreement
    pub payment_history: PaymentHistory<T>,
}

impl<T: pallet::Config> AgreementDetails<T> {
    /// Calculate the length of an installment. The length of the installment is the difference
    /// between the end block number of the installment and the end block number of the previous
    /// installment. The length of the first installment is the difference between the end block
    /// number of the first installment and the activation block.
    ///
    /// The installment index is the index of the installment in the payment plan. The first
    /// installment has index 0.
    ///
    /// Returns `None` if the installment index is out of bounds.
    fn calculate_installment_length(&self, installment_index: usize) -> Option<BlockNumberFor<T>> {
        let end_block = self.payment_plan.get(installment_index)?;
        let start_block = installment_index
            .checked_sub(1)
            .map(|i| self.payment_plan[i])
            .unwrap_or(self.activation_block);

        Some(end_block.saturating_sub(start_block))
    }

    /// Calculate the cost of an installment. The cost of the installment is the length of the
    /// installment multiplied by the storage cost per block.
    ///
    /// The installment index is the index of the installment in the payment plan. The first
    /// installment has index 0.
    ///
    /// Returns `None` if the installment index is out of bounds.
    fn calculate_installment_cost(&self, installment_index: usize) -> Option<BalanceOf<T>> {
        let installment_length = self.calculate_installment_length(installment_index)?;
        let cost = CurrentPrices::<T>::get()
            .storage_mb_per_block
            .saturating_mul(BalanceOf::<T>::saturated_from(
                installment_length.saturated_into::<u128>(),
            ));

        Some(cost)
    }

    /// Calculate the deposit amount for the consumer based on the payment plan. The deposit is
    /// the cost of the storage for the last installment.
    fn calculate_consumer_deposit(&self) -> BalanceOf<T> {
        self.calculate_installment_cost(self.payment_plan.len() - 1)
            .expect("empty payment plan is not allowed")
    }

    /// Create a new agreement with the status `ConsumerRequest`. The deposit is not calculated
    /// here, but it is calculated when the deposit is held.
    pub fn new_consumer_request(
        ip: AccountIdOf<T>,
        consumer: AccountIdOf<T>,
        storage: StorageSizeMB,
        activation_block: BlockNumberFor<T>,
        payment_plan: PaymentPlan<T>,
    ) -> Self {
        Self {
            ip,
            consumer,
            consumer_security_deposit: BalanceOf::<T>::zero(),
            consumer_security_deposit_transferred: false,
            status: AgreementStatus::ConsumerRequest,
            storage,
            activation_block,
            payment_plan,
            payment_history: PaymentHistory::new(),
        }
    }

    /// Update the status of the agreement and emit a corresponding event.
    pub fn update_status(&mut self, agreement_id: T::AgreementId, new_status: AgreementStatus) {
        self.status = new_status;

        Pallet::<T>::deposit_event(Event::AgreementStatusChanged {
            agreement_id,
            status: new_status,
        });
    }

    /// Holds the consumer deposit for the agreement. The deposit is the cost of the storage for the
    /// last installment. The deposit is calculated based on the payment plan and stored in the
    /// agreement.
    pub fn hold_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        let deposit = self.calculate_consumer_deposit();

        T::Currency::hold(
            &HoldReason::ConsumerSecurityDeposit.into(),
            &self.consumer,
            deposit,
        )?;

        self.consumer_security_deposit = deposit;
        Ok(deposit)
    }

    /// Releases the consumer deposit for the agreement. The deposit amount currently held is set to zero.
    pub fn release_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        let deposit = self.consumer_security_deposit;

        T::Currency::release(
            &HoldReason::ConsumerSecurityDeposit.into(),
            &self.consumer,
            deposit,
            Exact,
        )?;

        self.consumer_security_deposit = BalanceOf::<T>::zero();
        Ok(deposit)
    }

    /// Adjusts the consumer deposit for the agreement. This is called when the payment plan is
    /// changed. The deposit amount currently held is adjusted to the new deposit amount.
    /// The new deposit amount is calculated based on the new payment plan and stored in the
    /// agreement.
    ///
    /// Returns the new deposit amount.
    pub fn adjust_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        let current_deposit = self.consumer_security_deposit;
        let new_deposit = self.calculate_consumer_deposit();

        match current_deposit.cmp(&new_deposit) {
            Ordering::Less => T::Currency::hold(
                &HoldReason::ConsumerSecurityDeposit.into(),
                &self.consumer,
                new_deposit - current_deposit,
            ),
            Ordering::Greater => T::Currency::release(
                &HoldReason::ConsumerSecurityDeposit.into(),
                &self.consumer,
                current_deposit - new_deposit,
                Exact,
            )
            .map(|_| ()),
            Ordering::Equal => Ok(()),
        }?;

        self.consumer_security_deposit = new_deposit;
        Ok(new_deposit)
    }

    /// Transfers the consumer deposit to the IP.
    ///
    /// Returns the amount of the consumer deposit.
    pub fn transfer_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        T::Currency::transfer_on_hold(
            &HoldReason::ConsumerSecurityDeposit.into(),
            &self.consumer,
            &self.ip,
            self.consumer_security_deposit,
            Exact,
            Free,
            Polite,
        )?;

        self.consumer_security_deposit_transferred = true;
        Ok(self.consumer_security_deposit)
    }

    /// Holds the next installment for the agreement. The installment is calculated based on the
    /// payment plan and stored in the agreement's payment history.
    ///
    /// Returns the amount of the installment.
    pub fn hold_next_installment(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        let installment_index = self.payment_history.records.len();

        // Last installment is already paid with the consumer deposit
        ensure!(
            installment_index < self.payment_plan.len() - 1,
            Error::<T>::NoUnpaidInstallments
        );

        let installment_cost = self
            .calculate_installment_cost(installment_index)
            .ok_or(Error::<T>::NoUnpaidInstallments)?;

        T::Currency::hold(
            &HoldReason::ConsumerInstallment.into(),
            &self.consumer,
            installment_cost,
        )?;

        self.payment_history
            .records
            .try_push(PaymentRecord {
                amount: installment_cost,
                transferred: false,
            })
            .map_err(|_| ())
            .expect("payment history should never exceed the payment plan");

        Ok(installment_cost)
    }

    /// Transfers due installments to the IP. The installments are transferred from the consumer
    /// to the IP.
    ///
    /// Returns the total amount transferred.
    pub fn transfer_installments(
        &mut self,
        block_number: BlockNumberFor<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let current_installment = self.payment_history.next_transfer_installment_index as usize;

        // Calculate the total amount to be transferred and mark the installments as transferred
        let (mut total, count) = self
            .payment_history
            .records
            .iter_mut()
            .zip(self.payment_plan.iter())
            .skip(current_installment)
            .take_while(|(record, end_block)| !record.transferred && **end_block < block_number)
            .fold(
                (BalanceOf::<T>::zero(), 0),
                |(total, count), (record, _)| {
                    record.transferred = true;
                    (total.saturating_add(record.amount), count + 1)
                },
            );

        T::Currency::transfer_on_hold(
            &HoldReason::ConsumerInstallment.into(),
            &self.consumer,
            &self.ip,
            total,
            Exact,
            Free,
            Polite,
        )?;

        self.payment_history.next_transfer_installment_index += count;

        // Check if the agreement is complete and transfer the consumer deposit to the IP if it is
        if self.payment_plan.last() < Some(&block_number) {
            let deposit = self.transfer_consumer_deposit()?;
            total = total.saturating_add(deposit);
        }

        Ok(total)
    }

    /// Checks if there are any overdue installments. An installment is overdue if it is not
    /// prepaid by the consumer before the start of the installment. The last installment is
    /// always prepaid by the consumer deposit.
    pub fn has_overdue_installments(&self, block_number: BlockNumberFor<T>) -> bool {
        let next_unpaid_installment = self.payment_history.records.len();

        next_unpaid_installment < self.payment_plan.len() - 1
            && ((next_unpaid_installment == 0 && block_number >= self.activation_block)
                || (block_number > self.payment_plan[next_unpaid_installment - 1]))
    }
}

/// The current prices set by the network operator. The prices are:
/// - `storage_mb_per_block` - the rental cost of 1 MB of storage per block
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct Prices<T: pallet::Config> {
    /// Storage cost of 1 MB per block
    pub storage_mb_per_block: BalanceOf<T>,
}

impl<T: pallet::Config> Default for Prices<T> {
    fn default() -> Self {
        Self {
            storage_mb_per_block: Default::default(),
        }
    }
}
