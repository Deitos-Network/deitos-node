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

use frame_support::{
    assert_ok,
    pallet_prelude::*,
    parameter_types,
    traits::{ConstU32, ConstU64},
    PalletId,
};
use sp_core::H256;
use sp_keystore::{testing::MemoryKeystore, KeystoreExt};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate as pallet_deitos;
use crate::{CurrentAgreementId, IPStatus, PaymentPlan, StorageSizeMB};

mod agreements;
mod ip;
mod payments;
mod rating;

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Deitos: pallet_deitos,
    }
);

parameter_types! {
    pub const DeitosPalletId: PalletId = PalletId(*b"DeitosId");
}

type AccountId = u64;
type Balance = u64;
type AgreementId = u32;

pub const IP_INITIAL_DEPOSIT: Balance = 1_000_000;
pub const CONSUMER_SERVICE_DEPOSIT: Balance = 1_000;
pub const PRICE_STORAGE: Balance = 10;
pub const INITIAL_BALANCE: Balance = 1_000_000_000;
pub const IP: AccountId = 1;
pub const CONSUMER: AccountId = 2;

impl frame_system::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<3>;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = ();
    type WeightInfo = ();
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type MaxHolds = ConstU32<3>;
    type MaxFreezes = ();
}

impl pallet_deitos::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Currency = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type AgreementId = AgreementId;
    type PaymentPlanLimit = ConstU32<500>;
    type IPAgreementsLimit = ConstU32<500>;
    type ConsumerAgreementsLimit = ConstU32<500>;
    type PalletId = DeitosPalletId;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (1, INITIAL_BALANCE),
            (2, INITIAL_BALANCE),
            (3, INITIAL_BALANCE),
            (4, INITIAL_BALANCE),
        ],
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_deitos::GenesisConfig::<Test> {
        ip_initial_deposit: IP_INITIAL_DEPOSIT,
        consumer_service_deposit: CONSUMER_SERVICE_DEPOSIT,
        price_storage_mb_per_block: PRICE_STORAGE,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.register_extension(KeystoreExt::new(MemoryKeystore::new()));
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        if System::block_number() > 1 {
            System::on_finalize(System::block_number());
        }
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
    }
}

pub fn register_ip(ip: AccountId, total_storage: StorageSizeMB) {
    assert_ok!(Deitos::ip_register(
        RuntimeOrigin::signed(ip),
        total_storage
    ));
}

fn register_and_activate_ip(ip: AccountId, total_storage: StorageSizeMB) {
    register_ip(ip, total_storage);

    assert_ok!(Deitos::update_ip_status(
        RuntimeOrigin::root(),
        ip,
        IPStatus::Active
    ));
}

fn create_accepted_agreement(
    ip: AccountId,
    consumer: AccountId,
    storage: StorageSizeMB,
    activation_block: u64,
    payment_plan: PaymentPlan<Test>,
) -> AgreementId {
    assert_ok!(Deitos::consumer_request_agreement(
        RuntimeOrigin::signed(consumer),
        ip,
        storage,
        activation_block,
        payment_plan,
    ));

    let agreement_id = CurrentAgreementId::<Test>::get();
    assert_ok!(Deitos::ip_accept_agreement(
        RuntimeOrigin::signed(ip),
        agreement_id,
    ));

    agreement_id
}
