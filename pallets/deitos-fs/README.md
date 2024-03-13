# `pallet-deitos-fs` 

## 🚧🚧 Under Construction 🚧🚧

## Overview
The `pallet-deitos-fs` is designed to facilitate all the storage and file integrity management around agreements between consumers and providers.

### `pallet-deitos-fs` user guide.

The team behind the Deitos Network development has facilitated a user guide for interactacting from the functional perspective of this pallet. The documentation can be found [HERE](../../docs/pallet-deitos-user-guide.md).

### Processes Supported from `pallet-deitos-fs`


- **Data Integrity Protocol:** A mechanism designed to ensure the integrity and authenticity of the files stored within the infrastructure provider.

- **File Upload Verification Process:** This process involves the systematic verification of files post-upload to confirm their integrity and compliance with the terms of the user agreement. 

- **Deitos Verifier:** A dedicated component responsible for executing the file verification process. The Deitos Verifier operates as an off-chain worker that retrieves and validates file hashes, comparing them against the hashes recorded on the blockchain at the time of upload.

### Pallet extrinsics list

1. **register_file**: 
   - Description: Registers a a file under the status `Pending`. The file upload verification changes the status depending on the validation result.
   - Parameters: `agreement_id`, `hash` & `file_name`.

2. **ip_register**: 
   - Description: Registers an Infrastructure Provider (IP). The IP must not be already registered or must have been unregistered. It requires a deposit for registration, which is returned upon unregistration. The IP is initially registered with a 'Pending' status and must be activated by the network operator.
   - Parameters: `file_id`, `file` & `returned_hash`

3. **data_integrity_protocol**: 
   - Description: Emit the correponding event coming from the data integrity protocol result.
   - Parameters: `file_id`, `result`.

