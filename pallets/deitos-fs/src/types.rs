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

pub type FileName = BoundedVec<u8, ConstU32<64>>;

/// Lifecycle of a file
#[derive(Copy, Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub enum FileValidationStatus {
    /// FileStatus is being checked
    Pending,
    /// File was correctly validated
    Verified,
    /// It was a conflict during the initial check
    Conflict,
    /// To be removed,
    ToBeRemoved,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T: pallet::Config))]
pub struct FileDetails<T: pallet::Config> {
    /// Agreement ID
    pub agreement_id: T::AgreementId,
    /// MD5 of the file
    pub md5: [u8; 64],
    /// file name
    pub file_name: FileName,
    /// File validation status
    pub status: FileValidationStatus,
    /// Check error count
    pub error_count: u32,
}

impl<T: pallet::Config> FileDetails<T> {
    /// Create a new IP with the status `Pending`.
    pub fn new(agreement_id: T::AgreementId, md5: [u8; 64], file_name: FileName) -> Self {
        Self {
            file_name,
            agreement_id,
            md5,
            status: FileValidationStatus::Pending,
            error_count: 0,
        }
    }

    /// Updates the rating of the IP.
    pub fn update_status(&mut self, status: FileValidationStatus) {
        self.status = status;
    }
}

impl<T: pallet::Config> core::fmt::Debug for FileDetails<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileDetails {{ agreement_id: {:?}, md5: {:?}, file_name: {:?}, status: {:?}, error_count: {:?} }}", self.agreement_id, self.md5, self.file_name, self.status, self.error_count)
    }
}

pub enum OffChainType {
    /// File upload validation
    FileUpload,
    /// Data integrity check
    DataIntegrityCheck
}

