use super::*;
use frame_system::pallet_prelude::BlockNumberFor;

const EXPECTED_CONSUMER_BALANCE_LOCKED: u64 = 100; // 10% OF 1000

#[test]
fn test_successful_agreement_request_submission() {
    new_test_ext().execute_with(|| {
        let storage_size: StorageSizeMB = 100; // Example storage size in MB
        let activation_block: BlockNumberFor<Test> = 100; // Example activation block number

        // Mock proposed payment plan
        let proposed_plan: ProposedPlan<Test> = vec![
            (activation_block, activation_block + 100), // Sample plan entry
                                                        // Add more plan entries as needed
        ]
        .try_into()
        .expect("Failed to create proposed plan");

        // Register and activate the IP
        register_and_activate_ip(10000000);

        // Act: Call the submit_agreement_request function
        assert_ok!(Deitos::submit_agreement_request(
            RuntimeOrigin::signed(CONSUMER),
            IP,
            storage_size,
            activation_block,
            proposed_plan.clone(),
        ));

        // Assert: Verify that the agreement is correctly stored
        let stored_agreement = Agreements::<Test>::get(1).unwrap();
        assert_eq!(stored_agreement.consumer, CONSUMER);
        assert_eq!(stored_agreement.ip, IP);
        assert_eq!(stored_agreement.storage, storage_size);
        assert_eq!(stored_agreement.activation_block, activation_block);

        let expected_payment_plan_with_balance = vec![
            (activation_block, activation_block + 100, 1000), // Sample plan entry
                                                              // Add more plan entries as needed
        ];
        let expected_payment_plan: PaymentPlan<Test> =
            PaymentPlan::<Test>::try_from(expected_payment_plan_with_balance).unwrap();

        // print consumer balance
        let consumer_balance = Balances::free_balance(CONSUMER);

        assert_eq!(
            consumer_balance,
            INITIAL_BALANCE - EXPECTED_CONSUMER_BALANCE_LOCKED
        );

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerInitialDeposit.into(),
                &CONSUMER
            ),
            EXPECTED_CONSUMER_BALANCE_LOCKED
        );

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(
            pallet_deitos::Event::ConsumerRequestedAgreement {
                ip: IP,
                consumer: CONSUMER,
                status: AgreementStatus::ConsumerRequest,
                storage: storage_size,
                activation_block,
                payment_plan: expected_payment_plan, // or the expected payment plan after processing
            },
        ));
    });
}

/*
#[test]
fn test_successful_agreement_request() {
    new_test_ext().execute_with(|| {
        let consumer_account_id = 2;
        let ip_account_id = 1;
        let requested_storage: StorageSizeMB = 1000;
        let time_allocation: BlockNumberFor<Test> = 1000;
        let activation_block: PaymentPlan<Test> =  PaymentPlan<Test>::from(100);
        let payment_plan = PaymentPlan::Standard;

        // Register and activate an IP with sufficient storage
        let total_storage: StorageSizeMB = 10000000_u32.into();
        register_ip(total_storage);
        let ip_lookup = T::Lookup::unlookup(ip_account_id.clone());
        assert_ok!(Deitos::register_ip(RuntimeOrigin::root(), ip_lookup, IPStatus::Active));

        assert_ok!(Deitos::submit_agreement_request(
            RuntimeOrigin::signed(consumer_account_id),
            T::Lookup::unlookup(ip_account_id),
            requested_storage,
            time_allocation,
            activation_block,
            payment_plan.clone(),
        ));

        // Verify that the agreement is correctly stored
        let agreement_id = Deitos::next_agreement_id() - 1; // Assuming next_agreement_id() increments after each use
        let stored_agreement =
            Agreements::<Test>::get(agreement_id).expect("Agreement should be stored");

        assert_eq!(stored_agreement.consumer, consumer_account_id);
        assert_eq!(stored_agreement.ip, ip_account_id);
        assert_eq!(stored_agreement.storage, requested_storage);
        assert_eq!(stored_agreement.time_allocation, time_allocation);
        assert_eq!(stored_agreement.activation_block, activation_block);
        assert_eq!(stored_agreement.payment_plan, payment_plan);

        // Check for the correct event emission
        assert_has_event!(RuntimeEvent::Deitos(
            pallet_deitos::Event::ConsumerRequestedAgreement {
                ip: ip_account_id,
                consumer: consumer_account_id,
                status: AgreementStatus::ConsumerRequest,
                storage: requested_storage,
                time_allocation,
                activation_block,
                payment_plan,
            }
        ));
    });
}

#[test]
fn test_fail_agreement_request_inactive_ip() {
    new_test_ext().execute_with(|| {
        let consumer_account_id = 1;
        let ip_account_id = 2;
        let requested_storage: StorageSizeMB = 1000;
        let time_allocation = AgreementTimeAllocation::Monthly;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan = PaymentPlan::Standard;

        // Register IP without activating it
        let total_storage: StorageSizeMB = 10000000_u32.into();
        register_ip(ip_account_id, total_storage);

        // Attempt to submit agreement request and expect failure due to inactive IP
        assert_noop!(
            Deitos::submit_agreement_request(
                RuntimeOrigin::signed(consumer_account_id),
                T::Lookup::unlookup(ip_account_id),
                requested_storage,
                time_allocation,
                activation_block,
                payment_plan,
            ),
            Error::<Test>::IPNotActive
        );
    });
}

#[test]
fn test_fail_agreement_request_insufficient_storage() {
    new_test_ext().execute_with(|| {
        let consumer_account_id = 1;
        let ip_account_id = 2;
        let requested_storage: StorageSizeMB = 20000; // Greater than total storage
        let time_allocation = AgreementTimeAllocation::Monthly;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan = PaymentPlan::Standard;

        // Register an active IP with limited storage
        let total_storage: StorageSizeMB = 1000_u32.into();
        register_ip(ip_account_id, total_storage);
        make_ip_active(ip_account_id);

        // Attempt to submit agreement request and expect failure due to insufficient storage
        assert_noop!(
            Deitos::submit_agreement_request(
                RuntimeOrigin::signed(consumer_account_id),
                T::Lookup::unlookup(ip_account_id),
                requested_storage,
                time_allocation,
                activation_block,
                payment_plan,
            ),
            Error::<Test>::InsufficientStorage
        );
    });
}
 */
