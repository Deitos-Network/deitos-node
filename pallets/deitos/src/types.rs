#![allow(unused_qualifications)]

use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;

use crate::*;

pub type BalanceOf<T> =
    <<T as Config>::Currency as FunInspect<<T as frame_system::Config>::AccountId>>::Balance;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub type StorageSizeMB = u64;
pub type PaymentPlan<T> = BoundedVec<BlockNumberFor<T>, <T as Config>::PaymentPlanLimit>;
pub type IPAgreementsVec<T> =
    BoundedVec<<T as Config>::AgreementId, <T as Config>::IPAgreementsLimit>;
pub type ConsumerAgreementsVec<T> =
    BoundedVec<<T as Config>::AgreementId, <T as Config>::ConsumerAgreementsLimit>;

#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub enum IPStatus {
    Pending,
    Active,
    Unregistered,
}

#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub enum AgreementStatus {
    ConsumerRequest,
    IPProposedPaymentPlan,
    Agreed,
}

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

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct AgreementDetails<T: pallet::Config> {
    /// IP AccountId
    pub ip: AccountIdOf<T>,
    /// Consumer AccountId
    pub consumer: AccountIdOf<T>,
    /// Deposit amount payed by the consumer
    pub consumer_deposit: BalanceOf<T>,
    /// Agreement Status
    pub status: AgreementStatus,
    /// Total amount of storage in the agreement expressed in bytes?
    pub storage: StorageSizeMB,
    /// Activation block
    pub activation_block: BlockNumberFor<T>,
    /// Payment plan
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

    pub fn adjust_consumer_deposit(&mut self) -> Result<BalanceOf<T>, DispatchError> {
        let current_deposit = self.consumer_deposit;
        let new_deposit =
            Self::calculate_consumer_deposit(self.activation_block, &self.payment_plan);

        if current_deposit < new_deposit {
            T::Currency::hold(
                &HoldReason::ConsumerDeposit.into(),
                &self.consumer,
                new_deposit - current_deposit,
            )?;
        } else if current_deposit > new_deposit {
            T::Currency::release(
                &HoldReason::ConsumerDeposit.into(),
                &self.consumer,
                current_deposit - new_deposit,
                Exact,
            )?;
        }

        self.consumer_deposit = new_deposit;
        Ok(new_deposit)
    }
}

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
