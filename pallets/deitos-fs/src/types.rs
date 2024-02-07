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


use scale_info::TypeInfo;

use crate::*;

/// Type alias for the balance type from the runtime.
pub type BalanceOf<T> =
    <<T as Config>::Currency as FunInspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Type alias for `AccountId` from the runtime.
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// Type alias for `AccountId` lookup from the runtime.
pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;


/// Lifecycle of a file 
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
pub enum FileValidationStatus {
    /// FileStatus is being checked
    Pending,
    /// File was correctly validated
    Verified,
    /// It was a conflict during the initial check
    Conflict,
    /// To be removed,
    ToBeRemoved
}


/// File information
#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
pub struct FileDetails {
    /// MD5 of the file
    pub md5: [u8; 32],
    /// File validation status
    pub status: FileValidationStatus,
}

impl FileDetails {
    /// Create a new IP with the status `Pending`.
    pub fn new(md5: [u8; 32]) -> Self {
        Self {
            md5,
            status: FileValidationStatus::Pending
        }
    }
}

impl FileDetails {
    /// Updates the rating of the IP.
    pub fn update_status(&mut self, status: FileValidationStatus ) {
        self.status = status;
    }
}