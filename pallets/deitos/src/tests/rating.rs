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
    pallet::{Agreements, Event, HoldReason},
    types::*,
    InfrastructureProviders,
};

use super::*;

#[test]
fn test_consumer_submit_feedback() {
    new_test_ext().execute_with(|| {
        let storage: StorageSizeMB = 100;
        let activation_block: BlockNumberFor<Test> = 100;
        let payment_plan: PaymentPlan<Test> = vec![activation_block + 100].try_into().unwrap();

        register_and_activate_ip(IP, storage);
        let agreement_id = create_accepted_agreement(
            IP,
            CONSUMER,
            storage,
            activation_block,
            payment_plan.clone(),
        );

        run_to_block(activation_block + 101);
        assert_ok!(Deitos::ip_withdraw_installments(
            RuntimeOrigin::signed(IP),
            agreement_id,
        ));

        // Consumer submits feedback
        let comment = "Good service";
        assert_ok!(Deitos::consumer_submit_feedback(
            RuntimeOrigin::signed(CONSUMER),
            agreement_id,
            Score::Three,
            comment.to_owned(),
        ));

        // Verify that the agreement is removed
        assert_eq!(Agreements::<Test>::get(agreement_id), None);

        // Verify that the rating is stored
        let ip_details = InfrastructureProviders::<Test>::get(IP).unwrap();
        assert_eq!(
            ip_details.rating,
            Rating {
                cumulative_score: 3,
                number_of_ratings: 1
            }
        );

        // Verify that the consumer is refunded
        assert_eq!(
            Balances::free_balance(CONSUMER),
            INITIAL_BALANCE - 100 * PRICE_STORAGE
        );

        assert_eq!(
            <Balances as fungible::InspectHold<_>>::balance_on_hold(
                &HoldReason::ConsumerServiceDeposit.into(),
                &CONSUMER
            ),
            0
        );

        // Check for the correct event emission
        System::assert_has_event(RuntimeEvent::Deitos(Event::ConsumerSubmittedFeedback {
            agreement_id,
            consumer: CONSUMER,
            ip: IP,
            score: Score::Three,
            comment: comment.to_owned(),
        }));
    });
}
