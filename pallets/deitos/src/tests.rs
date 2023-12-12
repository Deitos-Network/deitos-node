use super::*;
use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

fn register_ip(total_storage: StorageSizeMB) {
    assert_ok!(Deitos::register_ip(RuntimeOrigin::signed(1), total_storage));
}

#[test]
fn test_correct_ip_registration() {
    new_test_ext().execute_with(|| {
        let total_storage: StorageSizeMB = 10000000_u32.into();
        let price_storage_per_block: BalanceOf<Test> = 1000_u64.into();

        register_ip(total_storage);

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::IPInitialDeposit.into(),
                &1
            ),
            DEPOSIT_AMOUNT
        );

        assert_eq!(
            <Balances as fungible::Inspect<_>>::balance(&1),
            INITIAL_BALANCE - DEPOSIT_AMOUNT
        );

        assert_eq!(
            InfrastructureProvider::<Test>::get(1).unwrap(),
            IPDetails::<Test> {
                total_storage,
                reserved_storage: Zero::zero(),
                status: IPStatus::Validating,
                active_agreements: BoundedVec::new(),
                deposit_amount: DEPOSIT_AMOUNT,
            }
        );

        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPRegistered {
            ip: 1,
            total_storage,
        }));
    });
}

#[test]
fn test_fail_ip_registration_already_exists() {
    new_test_ext().execute_with(|| {
        let ip = 1;
        let total_storage: StorageSizeMB = 10000000_u32.into();
        // Register the IP once
        register_ip(total_storage);

        // Attempt to register the same IP again and expect failure
        let total_storage: StorageSizeMB = 10000000_u32.into();
        assert_noop!(
            Deitos::register_ip(RuntimeOrigin::signed(ip), total_storage),
            Error::<Test>::IPAlreadyExists
        );
    });
}

#[test]
fn test_update_storage_cost_per_unit() {
    new_test_ext().execute_with(|| {
        // Arrange
        let new_price_per_block: BalanceOf<Test> = 2000_u64.into();

        // Act
        assert_ok!(Deitos::update_storage_cost_per_unit(
            RuntimeOrigin::root(),
            new_price_per_block,
        ));

        // Assert
        assert_eq!(
            IPUnitCosts::<Test>::get().unwrap().price_storage_per_block,
            new_price_per_block
        );

        System::assert_has_event(RuntimeEvent::Deitos(
            pallet_deitos::Event::StoragePriceUnitUpdated {
                price_storage_per_block: new_price_per_block,
            },
        ));
    });
}

#[test]
fn test_update_ip_status() {
    new_test_ext().execute_with(|| {
        let total_storage: StorageSizeMB = 10000000_u32.into();
        // Register the IP once
        register_ip(total_storage);
        // Arrange: Set up a registered IP and a status to update
        let ip = 1;
        let new_status = IPStatus::Active; // Replace with the correct enum value

        // Act: Update the IP status
        assert_ok!(Deitos::update_ip_status(
            RuntimeOrigin::root(),
            ip.into(), // Using `into()` to convert to AccountIdLookupOf<T>
            new_status.clone(),
        ));

        // Assert: Check if the IP status has been updated
        let ip_details = InfrastructureProvider::<Test>::get(ip).unwrap();
        assert_eq!(ip_details.status, new_status);

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(
            pallet_deitos::Event::IPStatusChanged {
                ip,
                status: new_status,
            },
        ));
    });
}

#[test]
fn test_unregister_ip() {
    new_test_ext().execute_with(|| {
        // Arrange: Register an IP first
        let ip = 1;
        let total_storage: StorageSizeMB = 10000000_u32.into();
        // Register the IP once
        register_ip(total_storage);

        // Check initial balance on hold for IP
        let initial_hold_balance = <Balances as fungible::InspectHold<_>>::balance_on_hold(
            &HoldReason::IPInitialDeposit.into(),
            &ip,
        );
        assert_eq!(initial_hold_balance, DEPOSIT_AMOUNT);

        // Act: Unregister the IP
        assert_ok!(Deitos::unregister_ip(RuntimeOrigin::signed(ip)));

        // Assert: Check if the IP is marked as unregistered
        let ip_details = InfrastructureProvider::<Test>::get(ip).unwrap();
        assert_eq!(ip_details.status, IPStatus::Unregistered);

        // Check balance after unregistering IP
        let balance_after_unregister = <Balances as fungible::Inspect<_>>::balance(&ip);
        assert_eq!(balance_after_unregister, INITIAL_BALANCE); // Assuming that the balance should be reset to initial balance

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPUnregistered {
            ip,
        }));
    });
}

// TODO - Add a test for `update_ip_status` to fail if there are ongoing agreements

#[test]
fn test_update_ip_storage() {
    new_test_ext().execute_with(|| {
        let ip = 1;
        let initial_storage: StorageSizeMB = 10000000_u32.into();
        // Register the IP once
        register_ip(initial_storage);

        let new_storage: StorageSizeMB = 5000000_u32.into();

        // Act: Update the storage capacity for the IP
        assert_ok!(Deitos::update_ip_storage(
            RuntimeOrigin::signed(ip),
            new_storage
        ));

        // Assert: Verify the storage capacity has been updated
        let ip_details = InfrastructureProvider::<Test>::get(ip).unwrap();
        assert_eq!(ip_details.total_storage, new_storage);

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(
            pallet_deitos::Event::IPStorageUpdated {
                ip,
                total_storage: new_storage,
            },
        ));
    });
}

#[test]
fn test_register_unregister_register_ip() {
    new_test_ext().execute_with(|| {
        let ip = 1;
        let total_storage: StorageSizeMB = 10000000_u32.into();
        // Register the IP once
        register_ip(total_storage);

        // Verify registration
        let ip_details_initial = InfrastructureProvider::<Test>::get(ip).unwrap();
        assert_eq!(ip_details_initial.status, IPStatus::Validating);

        // Step 2: Unregister the IP
        assert_ok!(Deitos::unregister_ip(RuntimeOrigin::signed(ip)));

        // Verify unregistration
        let ip_details_after_unreg = InfrastructureProvider::<Test>::get(ip).unwrap();
        assert_eq!(ip_details_after_unreg.status, IPStatus::Unregistered);

        // Step 3: Register the IP again
        register_ip(total_storage); // Re-register the same IP

        // Verify re-registration
        let ip_details_after_rereg = InfrastructureProvider::<Test>::get(ip).unwrap();
        assert_eq!(ip_details_after_rereg.status, IPStatus::Validating);

        // Assert: Check for the correct event emissions
        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPRegistered {
            ip,
            total_storage: 10000000_u32.into(),
        }));
        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPUnregistered {
            ip,
        }));
        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPRegistered {
            ip,
            total_storage: 10000000_u32.into(),
        }));
    });
}

#[test]
fn test_register_update_deposit_amount_unregister_ip() {
    new_test_ext().execute_with(|| {
        let ip = 1;
        let updated_deposit_amount: BalanceOf<Test> = 2000_u64.into(); // Example updated amount
        let total_storage: StorageSizeMB = 10000000_u32.into();
        // Register the IP once
        register_ip(total_storage);

        // Verify registration and initial deposit amount
        let initial_deposit = IPDepositAmount::<Test>::get().unwrap();
        assert_eq!(initial_deposit, DEPOSIT_AMOUNT);

        // Step 2: Update IPDepositAmount
        IPDepositAmount::<Test>::put(updated_deposit_amount);

        // Step 3: Unregister the IP
        assert_ok!(Deitos::unregister_ip(RuntimeOrigin::signed(ip)));

        // Verify that the original deposit amount is returned
        let final_balance = <Balances as fungible::Inspect<_>>::balance(&ip);
        assert_eq!(final_balance, INITIAL_BALANCE);

        // Assert: Check for the correct event emissions
        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPUnregistered {
            ip,
        }));
    });
}
