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
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_deitos::StorageSizeMB;

use crate::{
    pallet::{Event, Files},
    types::*,
};

fn to_hash(input: &str) -> [u8; 64] {
    let mut array = [0; 64]; // Initialize with zeros
    let bytes = input.as_bytes();
    array[..bytes.len().min(32)].copy_from_slice(&bytes[..64.min(bytes.len())]);
    array
}

pub fn create_agreement() {
    let storage: StorageSizeMB = 100;
    let activation_block: BlockNumberFor<Test> = 100;
    let payment_plan: PaymentPlan<Test> = vec![activation_block + 100].try_into().unwrap();

    // Register and activate the IP
    register_and_activate_ip(IP, storage);

    // Request agreement
    assert_ok!(Deitos::consumer_request_agreement(
        RuntimeOrigin::signed(CONSUMER),
        IP,
        storage,
        activation_block,
        payment_plan.clone(),
    ));

    // IP accepts agreement
    let agreement_id = 1;
    assert_ok!(Deitos::ip_accept_agreement(
        RuntimeOrigin::signed(IP),
        agreement_id,
    ));
}

#[test]
fn file_is_correctly_registered() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        let file_id = 1;
        create_agreement();

        let hash = to_hash("c43b3a108132702db1a3593550ef836081e781755dc32956c87c5be92e15d7c0");
        let file_name = b"file.txt".to_vec();

        assert_ok!(DeitosFs::register_file(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
            hash,
            file_name.into()
        ));

        // Verify that the agreement status is correctly updated
        let file = Files::<Test>::get(agreement_id).unwrap();
        assert_eq!(file.status, FileValidationStatus::Pending);
        assert_eq!(file.hash, hash);

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::DeitosFs(Event::FileRegistered {
            agreement_id,
            file_id,
            hash,
        }));
    });
}
