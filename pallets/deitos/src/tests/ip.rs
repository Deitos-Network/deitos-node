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

use frame_support::{assert_noop, traits::fungible};
use frame_system::pallet_prelude::BlockNumberFor;

use crate::{
    pallet::{CurrentPrices, Error, Event, HoldReason, IPDepositAmount, InfrastructureProviders},
    types::*,
};

use super::*;

#[test]
fn test_correct_ip_registration() {
    new_test_ext().execute_with(|| {
        let total_storage: StorageSizeMB = 10000000;

        register_ip(IP, total_storage);

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::IPInitialDeposit.into(),
                &IP
            ),
            IP_INITIAL_DEPOSIT
        );

        assert_eq!(
            <Balances as fungible::Inspect<_>>::balance(&IP),
            INITIAL_BALANCE - IP_INITIAL_DEPOSIT
        );

        assert_eq!(
            InfrastructureProviders::<Test>::get(IP).unwrap(),
            IPDetails::<Test> {
                total_storage,
                status: IPStatus::Pending,
                agreements: BoundedVec::new(),
                deposit: IP_INITIAL_DEPOSIT,
                rating: Rating {
                    cumulative_performance: 0,
                    cumulative_stability: 0,
                    cumulative_support: 0,
                    number_of_scores: 0,
                },
            }
        );

        System::assert_has_event(RuntimeEvent::Deitos(Event::IPRegistered {
            ip: IP,
            total_storage,
        }));
    });
}

#[test]
fn test_fail_ip_registration_already_exists() {
    new_test_ext().execute_with(|| {
        let total_storage: StorageSizeMB = 10000000;
        // Register the IP once
        register_ip(IP, total_storage);

        // Attempt to register the same IP again and expect failure
        assert_noop!(
            Deitos::ip_register(RuntimeOrigin::signed(IP), total_storage),
            Error::<Test>::IPAlreadyExists
        );
    });
}

#[test]
fn test_update_storage_cost_per_unit() {
    new_test_ext().execute_with(|| {
        // Arrange
        let new_price_per_block: BalanceOf<Test> = 2000;

        // Act
        assert_ok!(Deitos::update_storage_cost_per_unit(
            RuntimeOrigin::root(),
            new_price_per_block,
        ));

        // Assert
        assert_eq!(
            CurrentPrices::<Test>::get().storage_mb_per_block,
            new_price_per_block
        );

        System::assert_has_event(RuntimeEvent::Deitos(Event::StoragePriceUnitUpdated {
            price_storage_per_block: new_price_per_block,
        }));
    });
}

#[test]
fn test_update_ip_status() {
    new_test_ext().execute_with(|| {
        let total_storage: StorageSizeMB = 10000000;
        // Register the IP once
        register_ip(IP, total_storage);
        // Arrange: Set up a registered IP and a status to update
        let new_status = IPStatus::Active;

        // Act: Update the IP status
        assert_ok!(Deitos::update_ip_status(
            RuntimeOrigin::root(),
            IP,
            new_status,
        ));

        // Assert: Check if the IP status has been updated
        let ip_details = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details.status, new_status);

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPStatusChanged {
            ip: IP,
            status: new_status,
        }));
    });
}

#[test]
fn test_unregister_ip() {
    new_test_ext().execute_with(|| {
        // Arrange: Register an IP first
        let total_storage: StorageSizeMB = 10000000;
        // Register the IP once
        register_ip(IP, total_storage);

        // Check initial balance on hold for IP
        let initial_hold_balance = <Balances as fungible::InspectHold<_>>::balance_on_hold(
            &HoldReason::IPInitialDeposit.into(),
            &IP,
        );
        assert_eq!(initial_hold_balance, IP_INITIAL_DEPOSIT);

        // Act: Unregister the IP
        assert_ok!(Deitos::ip_unregister(RuntimeOrigin::signed(IP)));

        // Assert: Check if the IP is marked as unregistered
        let ip_details = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details.status, IPStatus::Unregistered);

        // Check balance after unregistering IP
        let balance_after_unregister = <Balances as fungible::Inspect<_>>::balance(&IP);
        assert_eq!(balance_after_unregister, INITIAL_BALANCE); // Assuming that the balance should be reset to initial balance

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPUnregistered { ip: IP }));
    });
}

#[test]
fn test_fail_ip_unregister_agreement_in_progress() {
    new_test_ext().execute_with(|| {
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

        // Accept agreement
        let expected_agreement_id = 1;
        assert_ok!(Deitos::ip_accept_agreement(
            RuntimeOrigin::signed(IP),
            expected_agreement_id
        ));

        // Attempt to unregister the IP and expect failure
        assert_noop!(
            Deitos::ip_unregister(RuntimeOrigin::signed(IP)),
            Error::<Test>::AgreementInProgress
        );
    });
}

#[test]
fn test_update_ip_storage() {
    new_test_ext().execute_with(|| {
        let initial_storage: StorageSizeMB = 10000000;
        // Register the IP once
        register_ip(IP, initial_storage);

        let new_storage: StorageSizeMB = 5000000;

        // Act: Update the storage capacity for the IP
        assert_ok!(Deitos::ip_update_storage(
            RuntimeOrigin::signed(IP),
            new_storage
        ));

        // Assert: Verify the storage capacity has been updated
        let ip_details = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details.total_storage, new_storage);

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPStorageUpdated {
            ip: IP,
            total_storage: new_storage,
        }));
    });
}

#[test]
fn test_register_unregister_register_ip() {
    new_test_ext().execute_with(|| {
        let total_storage: StorageSizeMB = 10000000;
        // Register the IP once
        register_ip(IP, total_storage);

        // Verify registration
        let ip_details_initial = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details_initial.status, IPStatus::Pending);

        // Step 2: Unregister the IP
        assert_ok!(Deitos::ip_unregister(RuntimeOrigin::signed(IP)));

        // Verify unregistration
        let ip_details_after_unreg = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details_after_unreg.status, IPStatus::Unregistered);

        // Step 3: Register the IP again
        register_ip(IP, total_storage); // Re-register the same IP

        // Verify re-registration
        let ip_details_after_rereg = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details_after_rereg.status, IPStatus::Pending);

        // Assert: Check for the correct event emissions
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPRegistered {
            ip: IP,
            total_storage: 10000000,
        }));
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPUnregistered { ip: IP }));
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPRegistered {
            ip: IP,
            total_storage: 10000000,
        }));
    });
}

#[test]
fn test_register_update_deposit_amount_unregister_ip() {
    new_test_ext().execute_with(|| {
        let updated_deposit_amount: BalanceOf<Test> = 2000; // Example updated amount
        let total_storage: StorageSizeMB = 10000000;
        // Register the IP once
        register_ip(IP, total_storage);

        // Verify registration and initial deposit amount
        let initial_deposit = IPDepositAmount::<Test>::get();
        assert_eq!(initial_deposit, IP_INITIAL_DEPOSIT);

        // Step 2: Update IPDepositAmount
        IPDepositAmount::<Test>::put(updated_deposit_amount);

        // Step 3: Unregister the IP
        assert_ok!(Deitos::ip_unregister(RuntimeOrigin::signed(IP)));

        // Verify that the original deposit amount is returned
        let final_balance = <Balances as fungible::Inspect<_>>::balance(&IP);
        assert_eq!(final_balance, INITIAL_BALANCE);

        // Assert: Check for the correct event emissions
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPUnregistered { ip: IP }));
    });
}
