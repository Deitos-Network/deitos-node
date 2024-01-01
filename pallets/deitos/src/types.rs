use core::cmp::Ordering;

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
/// `ConsumerRequest`. The IP can agree to the agreement and the status changes to `Agreed`, or
/// the IP can propose a payment plan and the status changes to `IPProposedPaymentPlan`. If the
/// consumer accepts the payment plan the status changes to `Agreed`.
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub enum AgreementStatus {
    /// Consumer requested an agreement
    ConsumerRequest,
    /// IP proposed a different payment plan
    IPProposedPaymentPlan,
    /// Agreement is agreed
    Agreed,
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

/// The details of an agreement. The agreement has:
/// - `ip` - the IP the agreement is with
/// - `consumer` - the consumer the agreement is with
/// - `consumer_deposit` - the deposit the consumer has payed to secure the agreement
/// - `status` - the current status of the agreement
/// - `storage` - the amount of storage covered by the agreement
/// - `activation_block` - the block number when the rental starts
/// - `payment_plan` - the payment plan for the agreement
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct AgreementDetails<T: pallet::Config> {
    /// IP participating in the agreement
    pub ip: AccountIdOf<T>,
    /// Consumer participating in the agreement
    pub consumer: AccountIdOf<T>,
    /// Deposit amount currently held for the agreement from the consumer
    pub consumer_deposit: BalanceOf<T>,
    /// Current status of the agreement
    pub status: AgreementStatus,
    /// The amount of storage covered by the agreement
    pub storage: StorageSizeMB,
    /// The block number when the rental starts
    pub activation_block: BlockNumberFor<T>,
    /// The payment plan for the agreement
    pub payment_plan: PaymentPlan<T>,
}

impl<T: pallet::Config> AgreementDetails<T> {
    /// Calculate the deposit amount for the consumer based on the payment plan. The deposit is
    /// the cost of the storage for the last installment.
    fn calculate_consumer_deposit(
        activation_block: BlockNumberFor<T>,
        payment_plan: &PaymentPlan<T>,
    ) -> BalanceOf<T> {
        let last_installment_length = match payment_plan.as_slice() {
            [.., prev, last] => last.saturating_sub(*prev),
            [single] => single.saturating_sub(activation_block),
            _ => unreachable!("empty payment plan is not allowed"),
        };

        CurrentPrices::<T>::get()
            .storage_mb_per_block
            .saturating_mul(BalanceOf::<T>::saturated_from(
                last_installment_length.saturated_into::<u128>(),
            ))
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
            consumer_deposit: BalanceOf::<T>::zero(),
            status: AgreementStatus::ConsumerRequest,
            storage,
            activation_block,
            payment_plan,
        }
    }

    /// Holds the consumer deposit for the agreement. The deposit is the cost of the storage for the
    /// last installment. The deposit is calculated based on the payment plan and stored in the
    /// agreement.
    pub fn hold_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        self.consumer_deposit =
            Self::calculate_consumer_deposit(self.activation_block, &self.payment_plan);

        T::Currency::hold(
            &HoldReason::ConsumerDeposit.into(),
            &self.consumer,
            self.consumer_deposit,
        )?;

        Ok(self.consumer_deposit)
    }

    /// Releases the consumer deposit for the agreement. The deposit amount currently held is set to zero.
    pub fn release_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        T::Currency::release(
            &HoldReason::ConsumerDeposit.into(),
            &self.consumer,
            self.consumer_deposit,
            Exact,
        )?;

        let deposit = self.consumer_deposit;
        self.consumer_deposit = BalanceOf::<T>::zero();
        Ok(deposit)
    }

    /// Adjusts the consumer deposit for the agreement. This is called when the payment plan is
    /// changed. The deposit amount currently held is adjusted to the new deposit amount.
    /// The new deposit amount is calculated based on the new payment plan and stored in the
    /// agreement.
    pub fn adjust_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        let current_deposit = self.consumer_deposit;
        let new_deposit =
            Self::calculate_consumer_deposit(self.activation_block, &self.payment_plan);

        match current_deposit.cmp(&new_deposit) {
            Ordering::Less => T::Currency::hold(
                &HoldReason::ConsumerDeposit.into(),
                &self.consumer,
                new_deposit - current_deposit,
            ),
            Ordering::Greater => T::Currency::release(
                &HoldReason::ConsumerDeposit.into(),
                &self.consumer,
                current_deposit - new_deposit,
                Exact,
            )
            .map(|_| ()),
            Ordering::Equal => Ok(()),
        }?;

        self.consumer_deposit = new_deposit;
        Ok(new_deposit)
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
