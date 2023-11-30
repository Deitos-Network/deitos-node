// Types required for pallet-deitos
use frame_system::pallet_prelude::BlockNumberFor;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type Storage = u64;
pub type AgreementTimeAllocation = u32;
pub type Installment = BalanceOf<T>;
pub type PaymentsDetails = (Installment, PaymentPlanPeriods);
pub type PaymentPlan = BoundedVec<PaymentsDetails<T>, MaxPaymentPlanDuration<T>>;



// TO-DO: Review the necessary status.
#[derive(Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub type IPStatus = enum {
    VALIDATING,
    ACTIVE,
    INACTIVE,
    SUSPENDED,
    TERMINATED,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub type AgreementStatus = enum {
    CONSUMER_REQUEST,
    IP_ACCEPTED,
    IP_ACCEPTED_WITH_MODIFICATIONS,
    IP_REJECTED,
    CONSUMER_ACCEPTED_MODIFICATIONS,
    ACTIVATED,
    DISPUTED,
    CONSUMER_BREACH,
    TERMINATED,
    WAITING_CONSUMER_FEEDBACK,
    WAITING_PROVIDER_FEEDBACK,
    FINISHED,
};

#[derive(Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub type PaymentPlanPeriods = enum {
    WEEKLY,
    BIWEEKLY,
    MONTHLY,
    QUARTERLY,
    SEMIANNUALLY,
    ANNUALLY
};


#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct InfraProviderDetails<T: pallet::Config> {
    // Price per block
    pub price_per_block: &BalanceOf<T>,
    // IP Status
    pub status: IPStatus,

}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct Agreement<T: pallet::Config> {
    // Agreement Status
    pub status: AgreementStatus,
    // Total amount of storage in the agreement expressed in bytes? 
    pub storage: Storage,
    // Amount of time the agreement is valid for in blocks
    pub time_allocation: AgreementTimeAllocation,
    // Activation block
    pub activation_block: BlockNumberFor<T>,
    // Payment plan
    pub payment_plan: PaymentPlan<T>,
    // Consumer agreement lock fee.
    pub agreement_lock_fee: BalanceOf<T>,
}
