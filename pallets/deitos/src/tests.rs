use super::*;
use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

fn register_ip() {
    let total_storage: Storage = 10000000_u32.into();
    assert_ok!(Deitos::register_ip(RuntimeOrigin::signed(1), total_storage,));
}

#[test]
fn test_correct_ip_registration() {
    new_test_ext().execute_with(|| {
        let total_storage: Storage = 10000000_u32.into();

        register_ip();

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
            IPDetails {
                total_storage,
                reserved_storage: Zero::zero(),
                status: IPStatus::Validating,
                active_agreements: BoundedVec::new(),
            }
        );

        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPRegistered {
            ip: 1,
            total_storage,
        }));
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
        register_ip();
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
        register_ip();

        // Act: Unregister the IP
        assert_ok!(Deitos::unregister_ip(RuntimeOrigin::signed(ip)));

        // Assert: Check if the IP is marked as unregistered
        let ip_details = InfrastructureProvider::<Test>::get(ip).unwrap();
        assert_eq!(ip_details.status, IPStatus::Unregistered); // Assuming 'Unregistered' is a status

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
        // Arrange: Register an IP and set initial storage
        let ip = 1;
        let initial_storage: Storage = 10000000_u32.into();
        Deitos::register_ip(RuntimeOrigin::signed(ip), initial_storage).unwrap();

        let new_storage: Storage = 5000000_u32.into();

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
