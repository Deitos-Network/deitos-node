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

use core::ops::Add;

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
    pub fn next_file_id() -> T::FileId {
        CurrentFileId::<T>::mutate(|value| {
            *value = value.add(One::one());
            *value
        })
    }
}
