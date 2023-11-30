// Types required for pallet-deitos

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


// TO-DO: Review the necessary status.
pub type IPStatus = enum {
    VALIDATING,
    ACTIVE,
    INACTIVE,
    SUSPENDED,
    TERMINATED,
}

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
pub struct Agreements<T: pallet::Config> {
    // Consumer account
    pub consumer: &T::AccountId,
}
