use core::ops::Add;

use frame_support::transactional;
use frame_system::pallet_prelude::BlockNumberFor;

use crate::*;

impl<T: Config> Pallet<T> {
    /// Deposits an event on a successful operation.
    pub fn success_event(event: Event<T>) -> DispatchResult {
        Self::deposit_event(event);
        Ok(())
    }

    /// Returns the current block number.
    pub fn current_block_number() -> BlockNumberFor<T> {
        frame_system::Pallet::<T>::block_number()
    }

    /// Generates the next agreement id and returns it.
    pub fn next_agreement_id() -> T::AgreementId {
        CurrentAgreementId::<T>::mutate(|value| {
            *value = value.add(One::one());
            *value
        })
    }

    /// Inserts a new agreement into the proper storages and returns the agreement id.
    #[transactional]
    pub fn insert_agreement(
        agreement: AgreementDetails<T>,
    ) -> Result<T::AgreementId, DispatchError> {
        let agreement_id = Self::next_agreement_id();

        InfrastructureProviders::<T>::try_mutate(&agreement.ip, |ip_details| {
            ip_details
                .as_mut()
                .ok_or(Error::<T>::IPNotFound)
                .and_then(|x| {
                    x.agreements
                        .try_push(agreement_id)
                        .map_err(|_| Error::<T>::IPAgreementsLimit)
                })
        })?;
        ConsumerAgreements::<T>::try_mutate(&agreement.consumer, |agreements| {
            agreements
                .try_push(agreement_id)
                .map_err(|_| Error::<T>::ConsumerAgreementsLimit)
        })?;

        Agreements::<T>::insert(agreement_id, agreement);
        Ok(agreement_id)
    }

    /// Deletes an agreement updating the proper storages and returns the agreement details.
    pub fn delete_agreement(
        agreement_id: T::AgreementId,
    ) -> Result<AgreementDetails<T>, DispatchError> {
        let agreement = Agreements::<T>::take(agreement_id).ok_or(Error::<T>::AgreementNotFound)?;
        InfrastructureProviders::<T>::mutate(&agreement.ip, |ip_details| {
            ip_details.as_mut().and_then(|x| {
                x.agreements
                    .iter()
                    .position(|&x| x == agreement_id)
                    .map(|i| x.agreements.swap_remove(i))
            })
        });
        ConsumerAgreements::<T>::mutate(&agreement.consumer, |agreements| {
            agreements
                .iter()
                .position(|&x| x == agreement_id)
                .map(|i| agreements.swap_remove(i))
        });
        Ok(agreement)
    }

    /// Check if the payment plan is valid. The payment plan is valid if the first installment
    /// block is in the future and the installments are strictly increasing.
    pub fn is_valid_payment_plan(
        payment_plan: &PaymentPlan<T>,
        activation_block: BlockNumberFor<T>,
    ) -> bool {
        payment_plan
            .first()
            .map(|end| *end > activation_block)
            .unwrap_or(false)
            && is_strictly_increasing(payment_plan)
    }
}
