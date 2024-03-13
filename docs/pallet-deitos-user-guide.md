# `pallet-deitos` User Guide


This user guide is structured as an interactive tutorial, designed to empower users to seamlessly navigate a range of essential processes within the [`pallet-deitos`](pallets/deitos) (core of Deitos orchestration) . Each step is elucidated with detailed explanations and visual aids, ensuring a comprehensive and user-friendly experience. The guide covers the following processes:

1. **Registering as an Infrastructure Provider:** A step-by-step guide on how to register as an Infrastructure Provider within the system.
2. **Submitting an Agreement Request as a Consumer:** Instructions on how consumers can initiate an agreement request, detailing the necessary parameters and procedures.
3. **Responding to Consumer Requests as an Infrastructure Provider:** Guidelines for Infrastructure Providers on how to address consumer requests, including accepting initial requests or proposing alternative payment plans.
4. **Managing Agreement Installments as a Consumer:** A comprehensive walkthrough for consumers on how to effectively manage and keep track of their agreement installments.
5. **Closing an Agreement and Submitting On-Chain Reputation:** Detailed steps on how to formally close an agreement and submit on-chain feedback for the Infrastructure Provider, contributing to their reputation.
6. **Renegotiating Agreement Payment Plans by the Infrastructure Provider:** Guidance on how Infrastructure Providers can initiate and negotiate changes to the payment plans of existing agreements.
7. **Handling Agreement Termination Due to Missed Consumer Payments:** Procedures and implications of terminating agreements due to non-payment by consumers, including the necessary steps and outcomes.
8. **Enacting Penalties for Missed Payments by Consumers:** Explanation of the protocols and penalties applied to consumers' reserved funds in cases of missed payments, outlining the consequences and enforcement mechanisms.


### Deitos Node Start

For detailed instructions on setting up and running the Deitos node, please refer to the [README.md](../README.md) file. 

## 1 - Infrastructure Provider Registration

### 1.1 - Register as an Infrastructure Provider

To register as an Infrastructure Provider (IP), execute the `ipRegister` extrinsic. This step involves registering BOB as an IP with a provision of 100 GB storage (expressed in MB as 10000). 

![userdoc.ip.register](assets/userdoc.ip.register.png)


The successful registration is confirmed by the `IPRegistered` event.

![userdoc.ip.event](assets/userdoc.ip.event.png)

A minimum balance of 10,000 DEITOs is required in the account for registration. This amount gets reserved from BOB's account upon submission of the `ipRegister` extrinsic, as shown in the account balance.

![userdoc.ip.balance](assets/userdoc.ip.balance.png)

### 1.2 - Manually Activate the IP

Initially, the IP status is `Pending`, indicating ongoing checks by the Deitos Protocol. To activate the IP manually, use the `updateIPStatus` extrinsic. This requires administrative access, typically available to the Root account (ALICE in local settings).

![userdoc.ip.storage](assets/userdoc.ip.storage.png)

The 'Pending' status signifies that the Deitos Protocol is in the process of thoroughly inspecting the Infrastructure Provider's technical stack. This phase involves a suite of checks and tests, meticulously conducted to confirm that all necessary services for the IP are functional and up to standard. As this validation process is still in its developmental stages, the interim solution involves using the `updateIPStatus` call. This permissioned action allows for the manual activation of the IP, facilitating the continuation of testing activities.

![IP Activation](assets/userdoc.ip.activation.png)

This step necessitates the use of the Sudo tab due to local settings where the Root key is configured with the ALICE account. Following these procedures, we will have a fully operational Infrastructure Provider, primed and ready to engage in agreements.

## 2 - Agreement's Submission (Happy Path)

### 2.1- Execute Agreement Request (Consumer)

Initiating the agreement negotiation process begins with the consumer submitting a request to the selected Infrastructure Provider (IP) using the `consumerRequestAgreement` extrinsic:

![userdoc.consumer.agreement.request](assets/userdoc.consumer.agreement.request.png)

Each parameter of the agreement request is detailed as follows:

- **IP:** The account of the chosen Infrastructure Provider.
- **Storage:** The total amount of storage allocated for use during the agreement.
- **Activation Block:** Given the negotiable nature of agreements, the activation block is set in the future.
- **Payment Plan:** This outlines the duration of each period within the agreement, specifying the start and end of each period expressed in block numbers.

The accompanying image outlines the agreement's periods as follows:

| Period | Start | End  | Duration | Installment             |
| ------ | ----- | ---- | -------- | ----------------------- |
| 1      | 50    | 80   | 30       | 30,000,000 (3 DEITOs)   |
| 2      | 80    | 110  | 30       | 30,000,000 (3 DEITOs)   |
| 3      | 110   | 140  | 30       | 30,000,000 (3 DEITOs)   |
| 4      | 140   | 160  | 20       | 20,000,000 (2 DEITOs)   |
 
Post-execution, the subsequent event confirms the request:

![userdoc.consumer.agreement.event](assets/userdoc.consumer.agreement.event.png)

To ensure the consumer's commitment to the agreement, the last installment (installment #4 in this scenario) is reserved from the consumer's account as a "service deposit". Additionally, a smaller "security deposit" is held, which is reimbursed upon the consumer's feedback submission following the agreement's conclusion. This security deposit is a fixed amount of 1,000,000 units (0.1 DEITO).

An inspection of the account will display the reserved balance as follows:

![userdoc.consumer.agreement.balance](assets/userdoc.consumer.agreement.balance.png)

### 2.2 - IP Agreement Request Acceptance

After the consumer submits their request, the Infrastructure Provider (IP) must decide to either accept or deny it. The IP also has the option to propose a new payment plan that better suits its needs. We will explore this in more detail later.

For now, the process of accepting an agreement using the `IpAcceptAgreement` extrinsic is straightforward:

![userdoc.ip.accept.agreement](assets/userdoc.ip.accept.agreement.png)

Execution of this extrinsic leads to a change in status, as depicted in the following events:

![userdoc.ip.accept.agreement.event](assets/userdoc.ip.accept.agreement.event.png)

### 2.3 - Consumer Installment Prepayment

With the agreement set to commence at block 50, the consumer is required to prepay the first installment. This step ensures that the first period of the agreement is financially secured:

![userdoc.consumer.agreement.prepay](assets/userdoc.consumer.agreement.prepay.png)

This prepayment action reserves the corresponding amount in the consumer’s account, meaning the funds are not immediately accessible to the IP. The installment becomes available for withdrawal by the IP after the period ends, as per our example, after block 80.

Upon reviewing CHARLIE's account, the reserved balance now includes the amount for the first installment:

![userdoc.consumer.agreement.balance.after](assets/userdoc.consumer.agreement.balance.after.png)

### 2.4 - Inspecting Agreement Storage Item

Inspecting the agreement storage item reveals comprehensive details about all actions related to the agreement:

![userdoc.ip.accept.agreement.storage](assets/userdoc.ip.accept.agreement.storage.png)

It also includes the payment history, showing that the first prepayment has been made and will become available to the IP after the end of the period.

### 2.5 - Completing All Installment Prepayments

To simplify this demonstration, we'll complete all prepayments, ensuring the consumer stays current with payments. This involves repeating the steps from 2.3 as necessary until all installments are paid.

Upon examining CHARLIE's account post-prepayment, the updated reserved balance is visible:

![userdoc.consumer.agreement.balance.allpaid](assets/userdoc.consumer.agreement.balance.allpaid.png)

### 2.6 - IP Withdrawal and Agreement Completion

Upon the conclusion of the agreement, the Infrastructure Provider (IP) is eligible to withdraw all corresponding installments for the service. The IP can withdraw the entire amount at once by executing the `IpWithdrawInstallments` extrinsic:

![userdoc.ip.withdraw](assets/userdoc.ip.withdraw.png)

The subsequent events detail the withdrawal process. With the agreement completed and no further payments due, the system automatically updates the agremeent's status to `Completed`:

![userdoc.ip.withdraw.event](assets/userdoc.ip.withdraw.event.png)

### 2.7 - IP Feedback Submission for On-Chain Reputation

With the agreement finalized, the last step in the process is the submission of feedback about the IP in relation to the agreement:

![userdoc.consumer.feedback](assets/userdoc.consumer.feedback.png)

The event showcases the submitted feedback:

![userdoc.consumer.feedback.event](assets/userdoc.consumer.feedback.event.png)

It's important to note that while the consumer can include comments in the feedback, which are displayed in the events, this text is not stored in any storage item. This reduce the need for unnecesary storage costs and still, having the text accessible by inspecting the events.

Following the feedback submission, the service deposit is released:

![userdoc.consumer.feedback.storage](assets/userdoc.consumer.feedback.storage.png)

### 2.8 - Agreement Cleanup

Anticipating a high volume of agreements, storage optimization measures have been implemented, which involve deleting storage items associated with finished agreements. To access historical agreement data, archive nodes and off-chain indexing services, akin to those used by block explorers, can be utilized.

## 3 Agreement Renegotiation

The deitos pallet is fundamentally designed to offer flexibility, enabling Infrastructure Providers (IPs) and consumers to negotiate mutually beneficial agreements. This flexibility allows for the modification of payment plans if the initial proposal does not meet the needs of both parties.

### 3.1 - Execute Agreement Request (Consumer)

In this scenario, we'll execute an agreement where the consumer sets a notably small security deposit, reflecting the comparatively lower amount of the final installment:

![userdoc.consumer.agreement.unbalance](assets/userdoc.consumer.agreement.unbalance.png)

### 3.2 - New Payment Plan Proposal

Suppose the IP prefers a more consistent payment plan with shorter intervals. In such a case, the IP has the option to propose a revised payment plan:

![userdoc.ip.new.payment.plan](assets/userdoc.ip.new.payment.plan.png)

The submission of this new payment plan is confirmed by the following event:

![userdoc.ip.new.payment.event.plan](assets/userdoc.ip.new.payment.plan.event.png)

### 3.3 - Consumer Rejects the New Payment Plan

If the consumer decides not to accept the new payment plan proposed by the Infrastructure Provider (IP), they have the option to revoke the agreement:

![userdoc.consumer.agreement.revoke](assets/userdoc.consumer.agreement.revoke.png)

The corresponding event confirms the revocation of the agreement. It also indicates that the deposit amount initially reserved for the agreement has been released and returned to the consumer:

![userdoc.consumer.agreement.revoke.event](assets/userdoc.consumer.agreement.revoke.event.png)

## 4 Agreements Cancelled Due to Unpaid Installment

This section demonstrates how an agreement is terminated if the consumer fails to continue payments. Non-payment leads to termination of the agreement and forfeiture of the consumer's deposits.

### 4.1 - Execute Agreement Request (Consumer)

As part of the process, the consumer requests an agreement, which is subsequently accepted by the Infrastructure Provider (IP):

![userdoc.consumer.request.tofail](assets/userdoc.consumer.request.tofail.png)

### 4.2 - Accept Agreement and Make First Installment

Once the IP accepts the agreement and it becomes active, the consumer proceeds to pay the first installment. This step is crucial for the agreement to commence as planned.


![userdoc.consumer.firstinstallment.tofail](assets/userdoc.consumer.firstinstallment.tofail.png)

Following the payment of the first installment, a specific balance is reserved from the consumer's account. This reserved amount includes the Service Deposit, the First Installment, and the Security Deposit (equivalent to the Last Installment).

![userdoc.consumer.balance.tofail](assets/userdoc.consumer.balance.tofail.png)

### 4.3 - Terminate the Agreement Due to Non-Payment

In this phase, the process involves deliberately not completing the consumer's payment, thereby placing the consumer at fault. When the IP observes that the payment has not been made within the designated timeframe, it has the authority to terminate the agreement and enact penalties against the consumer:

![userdoc.ip.terminate](assets/userdoc.ip.terminate.png)

As a result of this termination due to non-payment, the consumer faces penalties. All reserved funds associated with this agreement are transferred to the IP, and the agreement is conclusively terminated.

Following the termination and completion of all required actions, the agreement's related storage items are cleared from the system. This step ensures efficient management of system resources and data integrity.

## 5 Upload files and Data integrity protocol

Since we have interacted with every agreement's management option, now is time to focus on the file upload and data integrity protocol and let the Deitos protocol to validate if a consumer is granted or not to upload their files into the infrastructure provider.

As what we want to demonstrate here is the upload functionality of the protocol  we are going to re use the same infrastructure provider BOB, but using a different consumer this time: ALICE

For this we are going to create a simple agreement between ALICE (consumer) and BOB (IP) with only two very long installments  of 3000 blocks each, so we can focus on the file upload and verification:


## 5.1 Set agreement.

![userdoc.5.request.agreement](assets/userdoc.5.request.agreement.png)

Once confirmed, the event will retrieve the agreement id, which it will be necessary for next steps:

![userdoc.5.request.agreement.event](assets/userdoc.5.request.agreement.event.png)

Note: in this example, we are using agreement id "1", most probably the user following all the documentation will have higher number ids.

Similar as previous steps, the IP needs to approve the agreement:

![userdoc.5.ip.accept.agreement](assets/userdoc.5.ip.accept.agreement.png)

Finally, to make everything ready to upload files, the consumer should make an installment:

![userdoc.5.consumer.prepayinstalment](assets/userdoc.5.consumer.prepayinstalment.png)

Now, as we have everything ready so ALICE can start uploading files based on its agreement.

## 5.2 Uploading files.

Deitos development team has developed a CLI that allows the users to upload their files to the IP, based on all the agreements status.

For this, the user can download the following binary:


https://github.com/Deitos-Network/deitos-cli/releases/download/v0.0.1/deitos-cli

So once the binary was downloaded the can start the upload. 

First we create a test file:

```
 echo "this is my new super great file" > grant.txt
```

Then we invoke the CLI, with the followin command:

```
deitos-cli upload  --file-path=./examples/grant.txt --deitos-url=ws://localhost:9944 --ip-url=http://localhost:9090 --agreement=1 --suri=//Alice

```

As we can see, the file was correctly uploaded:

```
File hash: 4e9f6244c3edacc38d29792b9815803c64e4040106fad90771838f950968f072
File registration submitted, waiting for transaction to be finalized...
File registered successfully
Token data: Header { algorithm: "Sr25519", type_: Some(JsonWebToken) }
Claims { sub: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", aud: 1, exp: 1710354012001, iat: 1710267612001 }
Uploading file to http://localhost:9090/v1/files with JWT eyJhbGciOiJTcjI1NTE5IiwidHlwIjoiSldUIn0.eyJzdWIiOiI1R3J3dmFFRjV6WGIyNkZ6OXJjUXBEV1M1N0N0RVJIcE5laFhDUGNOb0hHS3V0UVkiLCJhdWQiOjEsImV4cCI6MTcxMDM1NDAxMjAwMSwiaWF0IjoxNzEwMjY3NjEyMDAxfQ.UhZb15pqZxk5zws1-8DyHcDLFgMMv4maKtHRIVz45GpcO1pvbpOGIB7v3GEycHZWo_UwT4nrA8K1jYrcREYAiw
File uploaded successfully
```

We will see an event confirming this:

![userdoc.5.file.registered](assets/userdoc.5.file.registered.png)


The CLI functioning is very interesting since many steps are involved within:

1) encrypts the file to SHA256 
2) In a JWT token, it wraps the signature of the consumer , in this case ALICE.
3) The Proxy (a.k.a Deitos Gate), reads the incomming request and it verifies if the consumer has that active agreement with that provider (in this case BOB).
4) If all conditions are met, it the proxy signs the message and invokes the extrinsic `deitosFS:registerFile`

We could validate the functioning of the CLI trying to upload a file from the wrong consumer:

First let's create another file:

```
echo "This file will never be uploaded" >  examples/file_to_fail.txt
```

Then trying to upload the file from another account like CHARLIE, we can see the Deitos Gate, refuses the request:

```
deitos-cli upload  --file-path=./examples/file_to_fail.txt --deitos-url=ws://localhost:9944 --ip-url=http://localhost:9090 --agreement=1 --suri=//Charlie
File hash: c0b4f8ec665aef7d05e5838b87010b1fdf8f3995ce21f312380d6e2c6ceb5c8a
File registration submitted, waiting for transaction to be finalized...
thread 'main' panicked at src/chain.rs:68:14:
File registration should finalize: Runtime(Module(ModuleError(<Deitos::NoAgreementForConsumer>)))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```



## 5.3  File upload verification

Once the file was uploaded, all the verification inside the IP starts. The following verification process starts on the IP side:

1) Query the recent uploaded file
2) Through the [Deitos Verifier](https://github.com/Deitos-Network/deitos-verifier), a off chain worker pulls the file content of the file, it calculate the corresponding SHA256 hash and if it match with the file hash calculated by the CLI, then the file is verified.

Once a file gets verified, the corresponding event will be triggered confirming this:


![userdoc.5.file.verified](assets/userdoc.5.file.verified.png)

Also, we could confirm the file was correctly uploaded by hitting the Hadoop file explorer:

![userdoc.5.fileuploaded.hadoop](assets/userdoc.5.fileuploaded.hadoop.png)
http://localhost:50070/explorer.html#/data/deitos

Accesing the file, we can check the file content matches :

![userdoc.5.fileuploaded.hadoop.content](assets/userdoc.5.fileuploaded.hadoop.content.png)

## 5.4 Data integrity protocol

Deitos Network checks the data integrity of the files uploaded. The principle is the same as the file upload verifier, by using the [Deitos Verifier](https://github.com/Deitos-Network/deitos-verifier) there is a off chain worker than during a random amount of blocks it picks randomnly a verified file and through the verifier, it assures that the hash registered in the blockchain matches with the file retrieved by the verifier.

As in this point of the exercise, we only have one file uploaded the data integrity protocol will report a successful check for the file ID 1:

![userdoc.5.file.check.verified](assets/userdoc.5.file.check.verified.png)



