use crate::*;

impl<T: Config> Pallet<T> {
    pub fn next_agreement_id() -> Result<T::AgreementId, sp_runtime::DispatchError> {
        let last_id = LastId::<T>::get().unwrap_or(Zero::zero());
        let payment_id: T::AgreementId = last_id.saturating_add(One::one());
        Ok(payment_id)
    }
}
