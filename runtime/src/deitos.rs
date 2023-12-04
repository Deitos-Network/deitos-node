use super::*;
use frame_support::PalletId;
/// Import the deitos pallet.
pub use pallet_deitos;
pub type AgreementId = u32;

parameter_types! {
    pub const DeitosPalletId: PalletId = PalletId(*b"DeitosId");
}

impl pallet_deitos::Config for Runtime {
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeEvent = RuntimeEvent;
    type AgreementId = AgreementId;
    type MaxPaymentPlanDuration = ConstU32<500>;
    type PalletId = DeitosPalletId;
    type WeightInfo = pallet_deitos::weights::SubstrateWeight<Runtime>;
}
