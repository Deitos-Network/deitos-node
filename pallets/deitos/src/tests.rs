use super::*;
use crate::{mock::*, types::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_correct_ip_registration() {
    new_test_ext().execute_with(|| {
        let total_storage: Storage = 10000000_u32.into();
        let price_storage_per_block: BalanceOf<Test> = 1000_u64.into();

        assert_ok!(Deitos::register_ip(
            RuntimeOrigin::signed(1),
            price_storage_per_block,
            total_storage,
        ));

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
                price_storage_per_block,
                total_storage,
                reserved_storage: Zero::zero(),
                status: IPStatus::Validating,
                active_agreements: BoundedVec::new()
            }
        );

        System::assert_has_event(RuntimeEvent::Deitos(pallet_deitos::Event::IPRegistered {
            ip: 1,
            price_storage_per_block,
            total_storage,
        }));
    });
}
