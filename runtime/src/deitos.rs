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

use super::*;
use frame_support::PalletId;
/// Import the deitos pallet.
pub use pallet_deitos;
pub use pallet_deitos_fs;

pub type AgreementId = u32;
pub type FileId = u32;

parameter_types! {
    pub const DeitosPalletId: PalletId = PalletId(*b"DeitosId");
}

impl pallet_deitos::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_deitos::weights::SubstrateWeight<Runtime>;
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type AgreementId = AgreementId;
    type PaymentPlanLimit = ConstU32<500>;
    type IPAgreementsLimit = ConstU32<500>;
    type ConsumerAgreementsLimit = ConstU32<500>;
    type PalletId = DeitosPalletId;
}

parameter_types! {
    pub const DeitosFsPalletId: PalletId = PalletId(*b"DeitosFs");
    pub const Seed: u32 = 12345;
    pub const ErrorBoundary: u32 = 200;
}

impl pallet_deitos_fs::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Currency = Balances;
    type FileId = FileId;
    type PalletId = DeitosFsPalletId;
    type Randomness = pallet_babe::ParentBlockRandomness<Runtime>;
    type Seed = Seed;
    type ErrorBoundary = ErrorBoundary;
}
