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

use frame_support::traits::fungible;
use frame_system::pallet_prelude::BlockNumberFor;

use crate::{
    pallet::{Agreements, ConsumerAgreements, Event, HoldReason, InfrastructureProviders},
    types::*,
};

use super::*;

#[test]
fn test_consumer_request_agreement() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![activation_block + 100, activation_block + 200]
            .try_into()
            .unwrap();

        // Register and activate the IP
        register_and_activate_ip(IP, storage);

        // Act: Request agreement
        assert_ok!(Deitos::consumer_request_agreement(
            RuntimeOrigin::signed(CONSUMER),
            IP,
            storage,
            activation_block,
            payment_plan.clone(),
        ));

        // Assert: Verify that the agreement request is correctly stored
        let expected_agreement_id = 1;
        let stored_agreement = Agreements::<Test>::get(expected_agreement_id).unwrap();
        let expected_consumer_deposit = 100 * PRICE_STORAGE; // Length of last installment * price per block
        assert_eq!(stored_agreement.ip, IP);
        assert_eq!(stored_agreement.consumer, CONSUMER);
        assert_eq!(stored_agreement.consumer_deposit, expected_consumer_deposit);
        assert_eq!(stored_agreement.status, AgreementStatus::ConsumerRequest);
        assert_eq!(stored_agreement.storage, storage);
        assert_eq!(stored_agreement.activation_block, activation_block);
        assert_eq!(stored_agreement.payment_plan, payment_plan);

        // Verify that the IP's data is correctly updated
        let ip_details = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details.agreements, vec![expected_agreement_id]);

        // Verify that the consumer's data is correctly updated
        let consumer_agreements = ConsumerAgreements::<Test>::get(CONSUMER);
        assert_eq!(consumer_agreements, vec![expected_agreement_id]);

        // Assert: Verify that the consumer's balance is properly updated
        assert_eq!(
            Balances::free_balance(CONSUMER),
            INITIAL_BALANCE - expected_consumer_deposit
        );

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerDeposit.into(),
                &CONSUMER
            ),
            expected_consumer_deposit
        );

        // Assert: Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::ConsumerRequestedAgreement {
            agreement_id: expected_agreement_id,
            ip: IP,
            consumer: CONSUMER,
            consumer_deposit: expected_consumer_deposit,
            storage,
            activation_block,
            payment_plan,
        }));
    });
}

#[test]
fn test_ip_accept_agreement() {
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

        // IP accepts agreement
        let agreement_id = 1;
        assert_ok!(Deitos::ip_accept_agreement(
            RuntimeOrigin::signed(IP),
            agreement_id,
        ));

        // Verify that the agreement status is correctly updated
        let stored_agreement = Agreements::<Test>::get(agreement_id).unwrap();
        assert_eq!(stored_agreement.status, AgreementStatus::Active);

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::IPAcceptedAgreement {
            agreement_id,
            ip: IP,
            consumer: CONSUMER,
        }));
    });
}

#[test]
fn test_consumer_accept_agreement() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![activation_block + 300].try_into().unwrap();
        let initial_consumer_deposit = 300 * PRICE_STORAGE;

        // Register and activate the IP
        register_and_activate_ip(IP, storage);

        // Consumer requests agreement with a payment plan of 1 installment (300 blocks)
        assert_ok!(Deitos::consumer_request_agreement(
            RuntimeOrigin::signed(CONSUMER),
            IP,
            storage,
            activation_block,
            payment_plan,
        ));

        // IP proposes payment plan of 2 installments (100, 200 blocks)
        let agreement_id = 1;
        let new_payment_plan: PaymentPlan<Test> =
            vec![activation_block + 100, activation_block + 300]
                .try_into()
                .unwrap();
        assert_ok!(Deitos::ip_propose_payment_plan(
            RuntimeOrigin::signed(IP),
            agreement_id,
            new_payment_plan.clone(),
        ));

        // Consumer accepts the new payment plan
        assert_ok!(Deitos::consumer_accept_agreement(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        // Verify that the agreement is correctly updated
        let expected_consumer_deposit = 200 * PRICE_STORAGE; // Length of last installment * price per block
        let stored_agreement = Agreements::<Test>::get(agreement_id).unwrap();
        assert_eq!(stored_agreement.consumer_deposit, expected_consumer_deposit);
        assert_eq!(stored_agreement.status, AgreementStatus::Active);
        assert_eq!(stored_agreement.payment_plan, new_payment_plan);

        // Verify that the consumer's balance is properly updated
        assert_eq!(
            Balances::free_balance(CONSUMER),
            INITIAL_BALANCE - expected_consumer_deposit
        );

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerDeposit.into(),
                &CONSUMER
            ),
            expected_consumer_deposit
        );

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::ConsumerAcceptedAgreement {
            agreement_id,
            ip: IP,
            consumer: CONSUMER,
            consumer_deposit_released: initial_consumer_deposit,
            consumer_deposit_held: expected_consumer_deposit,
        }));
    });
}

#[test]
fn consumer_reject_proposal() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![activation_block + 300].try_into().unwrap();
        let consumer_deposit = 300 * PRICE_STORAGE;

        // Register and activate the IP
        register_and_activate_ip(IP, storage);

        // Consumer requests agreement with a payment plan of 1 installment (300 blocks)
        assert_ok!(Deitos::consumer_request_agreement(
            RuntimeOrigin::signed(CONSUMER),
            IP,
            storage,
            activation_block,
            payment_plan,
        ));

        // IP proposes payment plan of 2 installments (100, 200 blocks)
        let agreement_id = 1;
        let new_payment_plan: PaymentPlan<Test> =
            vec![activation_block + 100, activation_block + 300]
                .try_into()
                .unwrap();
        assert_ok!(Deitos::ip_propose_payment_plan(
            RuntimeOrigin::signed(IP),
            agreement_id,
            new_payment_plan.clone(),
        ));

        // Consumer rejects the new payment plan
        assert_ok!(Deitos::consumer_revoke_agreement(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
        ));

        // Verify that the agreement is removed
        assert_eq!(Agreements::<Test>::get(agreement_id), None);

        // Verify that the IP's data is correctly updated
        let ip_details = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(ip_details.agreements, vec![]);

        // Verify that the consumer's data is correctly updated
        let consumer_agreements = ConsumerAgreements::<Test>::get(CONSUMER);
        assert_eq!(consumer_agreements, vec![]);

        // Verify that the consumer's balance is properly updated
        assert_eq!(Balances::free_balance(CONSUMER), INITIAL_BALANCE);

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerDeposit.into(),
                &CONSUMER
            ),
            0
        );

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::ConsumerRevokedAgreement {
            agreement_id,
            ip: IP,
            consumer: CONSUMER,
            consumer_deposit,
        }));
    });
}
