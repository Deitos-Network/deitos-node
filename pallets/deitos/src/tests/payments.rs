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
    pallet::{Agreements, Error, Event, HoldReason},
    types::*,
};

use super::*;

#[test]
fn test_consumer_prepay_installment() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![activation_block + 100, activation_block + 300]
            .try_into()
            .unwrap();

        register_and_activate_ip(IP, storage);
        let agreement_id = create_accepted_agreement(
            IP,
            CONSUMER,
            storage,
            activation_block,
            payment_plan.clone(),
        );

        let installment_cost = 100 * PRICE_STORAGE;
        let balance_before = Balances::free_balance(CONSUMER);

        // Consumer prepays the first installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        // Verify that the agreement is correctly updated
        let stored_agreement = Agreements::<Test>::get(agreement_id).unwrap();
        assert_eq!(
            stored_agreement.payment_history.records,
            vec![PaymentRecord {
                amount: installment_cost,
                transferred: false
            }]
        );

        // Verify that the consumer's balance is properly updated
        assert_eq!(
            Balances::free_balance(CONSUMER),
            balance_before - installment_cost
        );

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerInstallment.into(),
                &CONSUMER
            ),
            installment_cost
        );

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::ConsumerPrepaidInstallment {
            agreement_id,
            consumer: CONSUMER,
            cost: installment_cost,
        }));
    });
}

#[test]
fn test_consumer_prepay_multiple() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![
            activation_block + 100,
            activation_block + 300,
            activation_block + 600,
        ]
        .try_into()
        .unwrap();

        register_and_activate_ip(IP, storage);
        let agreement_id = create_accepted_agreement(
            IP,
            CONSUMER,
            storage,
            activation_block,
            payment_plan.clone(),
        );

        let balance_before = Balances::free_balance(CONSUMER);

        // Consumer prepays the first installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        // Consumer prepays the second installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        // Verify that the agreement is correctly updated
        let expected_payment_records = vec![
            PaymentRecord {
                amount: 100 * PRICE_STORAGE,
                transferred: false,
            },
            PaymentRecord {
                amount: 200 * PRICE_STORAGE,
                transferred: false,
            },
        ];
        let stored_agreement = Agreements::<Test>::get(agreement_id).unwrap();
        assert_eq!(
            stored_agreement.payment_history.records,
            expected_payment_records
        );

        // Verify that the consumer's balance is properly updated
        let total_cost = 300 * PRICE_STORAGE;
        assert_eq!(
            Balances::free_balance(CONSUMER),
            balance_before - total_cost
        );

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerInstallment.into(),
                &CONSUMER
            ),
            total_cost
        );

        // Verify that consumer cannot prepay any more installments
        assert_noop!(
            Deitos::consumer_prepay_installment(RuntimeOrigin::signed(CONSUMER), agreement_id),
            Error::<Test>::NoUnpaidInstallments
        );
    });
}

#[test]
fn test_ip_withdraw() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![
            activation_block + 100,
            activation_block + 300,
            activation_block + 600,
        ]
        .try_into()
        .unwrap();

        register_and_activate_ip(IP, storage);
        let agreement_id = create_accepted_agreement(
            IP,
            CONSUMER,
            storage,
            activation_block,
            payment_plan.clone(),
        );

        // Consumer prepays the first installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        // Consumer prepays the second installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        let balance_before = Balances::free_balance(IP);

        // IP withdraws the first installment (the second one is not yet due)
        run_to_block(activation_block + 101);
        assert_ok!(Deitos::ip_withdraw_installments(
            RuntimeOrigin::signed(IP),
            agreement_id,
        ));

        // Verify that the agreement is correctly updated
        let expected_payment_records = vec![
            PaymentRecord {
                amount: 100 * PRICE_STORAGE,
                transferred: true,
            },
            PaymentRecord {
                amount: 200 * PRICE_STORAGE,
                transferred: false,
            },
        ];
        let stored_agreement = Agreements::<Test>::get(agreement_id).unwrap();
        assert_eq!(
            stored_agreement.payment_history.records,
            expected_payment_records
        );
        assert_eq!(
            stored_agreement
                .payment_history
                .next_transfer_installment_index,
            1
        );

        // Verify that the IP's balance is properly updated
        let total_cost = 100 * PRICE_STORAGE;
        assert_eq!(Balances::free_balance(IP), balance_before + total_cost);

        //Verify that the consumer's balance is properly updated
        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerInstallment.into(),
                &CONSUMER
            ),
            200 * PRICE_STORAGE
        );

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPWithdrewInstallments {
            agreement_id,
            ip: IP,
            transferred: total_cost,
        }));
    });
}

#[test]
fn test_ip_withdraw_completely() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![
            activation_block + 100,
            activation_block + 300,
            activation_block + 600,
        ]
        .try_into()
        .unwrap();

        register_and_activate_ip(IP, storage);
        let agreement_id = create_accepted_agreement(
            IP,
            CONSUMER,
            storage,
            activation_block,
            payment_plan.clone(),
        );

        // Consumer prepays the first installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        // Consumer prepays the second installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        let balance_before = Balances::free_balance(IP);

        // IP withdraws all installments
        run_to_block(activation_block + 601);
        assert_ok!(Deitos::ip_withdraw_installments(
            RuntimeOrigin::signed(IP),
            agreement_id,
        ));

        // Verify that the agreement is correctly updated
        let stored_agreement = Agreements::<Test>::get(agreement_id).unwrap();
        assert!(stored_agreement.consumer_security_deposit_transferred);
        assert_eq!(stored_agreement.status, AgreementStatus::Completed);

        // Verify that the IP's balance is properly updated
        let total_cost = 600 * PRICE_STORAGE;
        assert_eq!(Balances::free_balance(IP), balance_before + total_cost);

        //Verify that the consumer's balance is properly updated
        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerInstallment.into(),
                &CONSUMER
            ),
            0
        );

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::AgreementStatusChanged {
            agreement_id,
            status: AgreementStatus::Completed,
        }));
    });
}

#[test]
fn test_ip_terminate_nonpay() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![
            activation_block + 100,
            activation_block + 300,
            activation_block + 600,
        ]
        .try_into()
        .unwrap();

        register_and_activate_ip(IP, storage);
        let agreement_id = create_accepted_agreement(
            IP,
            CONSUMER,
            storage,
            activation_block,
            payment_plan.clone(),
        );

        // Consumer prepays the first installment
        assert_ok!(Deitos::consumer_prepay_installment(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        let balance_before = Balances::free_balance(IP);

        // IP terminates the agreement because the second installment is not prepaid
        run_to_block(activation_block + 101);
        assert_ok!(Deitos::ip_terminate_nonpay(
            RuntimeOrigin::signed(IP),
            agreement_id,
        ));

        // Verify that the agreement is removed
        assert_eq!(Agreements::<Test>::get(agreement_id), None);

        // Verify that the IP's balance is properly updated
        let transferred = 400 * PRICE_STORAGE + CONSUMER_SERVICE_DEPOSIT;
        assert_eq!(Balances::free_balance(IP), balance_before + transferred);

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPTerminatedNonPay {
            agreement_id,
            ip: IP,
            transferred,
        }));
    });
}
