use super::*;
use frame_support::PalletId;
/// Import the deitos pallet.
pub use pallet_deitos;
pub type AgreementId = u32;

parameter_types! {
    pub const DeitosPalletId: PalletId = PalletId(*b"DeitosId");
}

impl pallet_deitos::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type WeightInfo = pallet_deitos::weights::SubstrateWeight<Runtime>;
    type AgreementId = AgreementId;
    type PaymentPlanLimit = ConstU32<500>;
    type IPAgreementsLimit = ConstU32<500>;
    type ConsumerAgreementsLimit = ConstU32<500>;
    type PalletId = DeitosPalletId;
}
