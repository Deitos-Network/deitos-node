# `pallet-deitos` 

## 🚧🚧 Under Construction 🚧🚧

## Overview
The `pallet-deitos` is a fundamental component of the Deitos Network blockchain node. It's designed to facilitate the hearth of the Deitos Network.



### Directory Structure
- `src/`: The main source code directory.
  - `benchmarking.rs`: Contains benchmarking for pallet extrinsics (WIP).
  - `impls.rs`: Implementation details of specific traits or functionalities.
  - `lib.rs`: The core library file, serving as the entry point for the pallet.
  - `types.rs`: Defines various types and implementations used within the pallet.
  - `weights.rs`: Contain weights resulting from the benchmarking execution (WIP).
- `tests/`: Contains test cases for the pallet.
  - `agreements.rs`: Tests related to agreement functionalities.
  - `ip.rs`: Tests for infrastructure providers features.
  - `mod.rs`: Module file for tests.
  - `payments.rs`: Tests for payment-related functionalities within the deitos agreeemnts context.
  - `rating.rs`: Tests for rating systems or mechanisms used in On-Chain reputation.

### `pallet-deitos` user guide.

The team behind the Deitos Network development has facilitated a user guide for interactacting from the functional perspective of this pallet. The documentation can be found [HERE](../../docs/pallet-deitos-user-guide.md).

### Processes Supported from `pallet-deitos`

The `pallet-deitos` supports a range of functionalities central to the orchestration of agreements between Infrastructure Providers (IPs) and consumers within the Deitos ecosystem. These functionalities are designed to facilitate seamless interactions, ensuring efficient management and execution of agreements. The key processes supported by `pallet-deitos` include:

- **Infrastructure Provider Management**:
  - **Registration**: Enables entities to register as Infrastructure Providers. This process involves executing the `ipRegister` extrinsic and maintaining a minimum balance requirement.
  - **Resource Allocation**: After registration, IPs can allocate resources such as storage space.
  - **Status Activation**: IPs initially have a 'Pending' status, which can be manually activated to 'Active' using the `updateIPStatus` extrinsic, indicating readiness to engage in agreements.

- **Agreements Functionality**:
  - **Agreement Initiation**: Consumers can initiate agreements with IPs specifying parameters like storage amount and payment plan.
  - **Agreement Acceptance**: IPs have the ability to accept consumer-initiated agreements.
  - **Installment Management**: Consumers are required to prepay installments for the agreement, which are reserved in their account and later accessible to the IP.
  - **Agreement Renegotiation**: Both IPs and consumers can negotiate the terms of the agreement. IPs can propose new payment plans, and consumers have the option to accept or revoke the agreement.
  - **Agreement Termination**: Agreements can be terminated due to non-payment by the consumer, with penalties applied and reserved funds transferred to the IP.
  - **Feedback and Reputation**: Post-agreement, consumers can submit feedback about IPs, contributing to their on-chain reputation.

- **Financial Transactions and Penalties**:
  - **Prepayment and Withdrawals**: Consumers prepay installments, and IPs can withdraw these funds post the installment period.
  - **Penalties for Non-Payment**: In case of agreement termination due to non-payment, penalties are enforced, and reserved funds are allocated to the IP.

- **Storage and Data Management**:
  - **Agreement Storage Inspection**: Detailed information about agreements, including payment history, is available in the agreement storage item.
  - **Storage Optimization**: Completed agreements are cleaned up from storage to optimize space and maintain system efficiency.

### Pallet extrinsics list
The Deitos pallet includes several key features, each corresponding to different aspects of the Deitos Network's functionality. Below is a brief description of each extrinsic:

1. **ip_register**: 
   - Description: Registers an Infrastructure Provider (IP). The IP must not be already registered or must have been unregistered. It requires a deposit for registration, which is returned upon unregistration. The IP is initially registered with a 'Pending' status and must be activated by the network operator.
   - Parameters: `total_storage`.

2. **update_ip_status**: 
   - Description: Updates the status of an IP. This function can only be called by the `Root` origin. It changes the status of an IP in the network.
   - Parameters: `ip`, `status`.

3. **ip_update_storage**: 
   - Description: Updates the total storage of an IP. This function allows an IP to update the amount of storage it offers.
   - Parameters: `total_storage`.

4. **ip_unregister**: 
   - Description: Unregisters an IP. The IP must be registered and must not have any ongoing agreements. The deposit paid during registration is returned upon successful unregistration.
   - Parameters: `origin`.

5. **update_storage_cost_per_unit**: 
   - Description: Updates the price for storage per block. This can only be done by the network operator and does not affect installments that have already been paid.
   - Parameters: `price_storage_per_block`.

6. **consumer_request_agreement**: 
   - Description: Allows a consumer to request an agreement with an IP. The IP must be registered and active. The consumer must pay a deposit to secure the agreement. The function requires details like the amount of storage needed, the start block number, and the payment plan.
   - Parameters: `ip`, `storage`, `activation_block`, `payment_plan`.

7. **consumer_revoke_agreement**: 
   - Description: Revokes an agreement that is in progress. This can only be done by the consumer who initiated the agreement. The deposit paid to secure the agreement is returned upon revocation.
   - Parameters: `agreement_id`.

8. **ip_accept_agreement**: 
   - Description: Accept an agreement by the IP that the agreement is with. The activation block must not be in the past. The status of the agreement must be `ConsumerRequest`. The agreement is accepted by the IP and the status changes to `Active`.
   - Parameters: `agreement_id`.

9. **ip_propose_payment_plan**: 
   - Description: Propose a new payment plan for an agreement. The agreement status must be `ConsumerRequest`.
   - Parameters: `agreement_id`, `payment_plan`

10. **consumer_accept_agreement**: 
   - Description: Accept a payment plan proposed by an IP. The agreement status must be `IPProposedPaymentPlan`. The consumer deposit is adjusted to the new payment plan. The agreement status changes to `Active`.
   - Parameters: `agreement_id`, `payment_plan`

11. **consumer_prepay_installment**: 
   - Description: Prepay an installment. The agreement status must be `Active`. The consumer pays the cost of the next unpaid installment. All payments are saved in the agreement's payment history.
   - Parameters: `agreement_id`

12. **ip_withdraw_installments**: 
   - Description: Withdraw installments. The agreement status must be `Active`. The IP withdraws all complete installments from the agreement. The IP can withdraw installments only if the consumer has prepaid them. If the agreement is fully paid, the status changes to `Completed`.
   - Parameters: `agreement_id`

13. **ip_terminate_nonpay**: 
   - Description: Terminate an agreement due to non-payment. The agreement status must be `Active`. The IP receives all unpaid installments and the consumer deposit. The agreement is deleted.
   - Parameters: `agreement_id`

14. **consumer_submit_feedback**: 
   - Description: Submit feedback for an agreement. The agreement status must be `Completed`. The consumer submits a score and a comment. The completed agreement is deleted. The consumer service deposit is released.
   - Parameters: `agreement_id`, `score_performance`, `score_stability`, `score_support`, `comment`

### Deitos Agreements flow

![agreements.flow](../../docs/assets/agreements.flow.jpg)