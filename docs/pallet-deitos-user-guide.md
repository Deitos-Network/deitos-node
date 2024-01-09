# `pallet-deitos` User Guide

This documentation contains a comprehensive guide to perform all the available interactions within the [`pallet-deitos`](pallets/deitos).

This user guide will be developed in a tutorial format, allowing to the user to interact with the following proceses:

- Infrastructure Provider registration.
- Submit an agreement request as a consumer. 
- Answer to the consumer's request as an Infrastructure Provider by accepting the initial request or by proposing a new payment plan.
- Operate and manage agreement's installments as a consumer.
- Close an agreement and submit the on chain reputation for the infrastrucure provider.



## 1 - Infrasctructure Provider Registration

For this test we will use BOB as the corresponding account to register the Infrastructure Provider.

In order to commit the registation, the infrastructure provider the following sequence should be executed:

### 1.1) Register Register

In order to start the registration for the IP the `ipRegister` extrinsic should be executed:

![userdoc.ip.register](assets/userdoc.ip.register.png)

This extrinsic will make the submission to register BOB as an IP indicating the infrastructure provider will provide 100 GB storage, expressed in MB (10000). The `IPRegistered` event will reflect this action:

![userdoc.ip.event](assets/userdoc.ip.event.png)

In order to submit the intent of become an IP in the Deitos Network, the account should contain at least the minumum balance of 10.000 DEITOs. So, as soon as the `ipRegister` extrinsic is submitted, this amount is reserved from BOBs account. This can be checked by quering the account balance:

![userdoc.ip.balance](assets/userdoc.ip.balance.png)

### 1.2) Manually activate the IP as root.

If we inspect the value of the `InfrastructureProviders` storage item for BOB, we can check that the IP status is `Pending`:

![userdoc.ip.storage](assets/userdoc.ip.storage.png)


This status means that all the tech stack for the IP is currently being checked by the Deitos Protocol. During the validation, the Deitos protocol will trigger a set of tests that will validate that all the required services for the IP are up and running
As this validation is not developed yet, in order to activate the IP we will rely on the permissioned call `updateIPStatus` to continue with the test:

![userdoc.ip.activation](assets/userdoc.ip.activation.png)

This will require using the Sudo tab, that because of the local settings the Root key was configured with ALICE account.

Up to this point we should have a fully functional and ready IP to accept agreements!

## 2 - Agreement's submission (happy path)

Now we have an infrastructure provider up and running, the next step will be to submit an agreement request taking the consumer role. We will use the CHARLIE test account to play the role of a consumer.

### 2.1) Execute agreement's request (Consumer)

The first step to start the agreement negotiation process is that a consumer submits an agreement request to the selected IP. 

![userdoc.consumer.agreement.request](assets/userdoc.consumer.agreement.request.png)

Let's review each parameter in detail:

Ip: Account for the selected infrastructure Provider.
Storage: The amount of storage to be used during the agreement. 
Activation block: As agreements can be negotiated, the activation block has to be set in the future. 
Payment Plan: This defines the duration of each period during the agreement indicating the beggining and end of each period. 

In the image attached, that would define the following periods for that agreement:


| Period | Start | End  | Duration | Installment |
|--------|-------|------|----------|-------------|
| 1      | 50    | 80   | 30       | 30,000,000 (3 DEITOs)  
| 2      | 80    | 110  | 30       | 30,000,000 (3 DEITOs) 
| 3      | 110   | 140  | 30       | 30,000,000 (3 DEITOs)  
| 4      | 140   | 160  | 20       | 20,000,000 (2 DEITOs) 

Once this is executed the following event would confirm the request:

![userdoc.consumer.agreement.event](assets/userdoc.consumer.agreement.event.png)

In order to assure that consumer will honor the agreement with the IP the last installment (in this case installment #4) is already locked from consumer's account as a "service deposit". Also there is a smaller deposit called "security deposit" which will be returned once the consumer submits the feedback after an agreement termination. This last deposit is a fixed amount of 1,000,000 units (0.1 DEITO)

If the account is inspected, we will see the reserved balance accordingly:

![userdoc.consumer.agreement.balance](assets/userdoc.consumer.agreement.balance.png)


### 2.2) IP agreement request acceptance.

Once the request was submitted by the consumer, naturally the IP have to accept or deny the request. Also the IP is capable to propose a new payment plan that adjust better to its need, but we will leave that case for later.

For the moment the extrinsic `IpAcceptAgreement`  to accept the agreement is quite simple:

![userdoc.ip.accept.agreement](assets/userdoc.ip.accept.agreement.png)

Once this is executed we will be able to see the status changes on the two events listed:

![userdoc.ip.accept.agreement.event](assets/userdoc.ip.accept.agreement.event.png)

### 2.3) Consumer installment prepayment 

As the agreement is ready to start in the block 50, the consumer should commit the first installment prepay so the first period is financially covered:

![userdoc.consumer.agreement.prepay](assets/userdoc.consumer.agreement.prepay.png)

This prepayment reserves the balance on the consumer account, which means that the funds are not available for the IP yet. This installment will be available to withdraw after the period is finished, following the example after the block 80.

If we inspect CHARLIE account we will see that the reserved balance now adds the first installment amount:

![userdoc.consumer.agreement.balance.after](assets/userdoc.consumer.agreement.balance.after.png)

### 2.4) Inspecting agreement storage item

And if we inspect the agreement storage item, we will see all the detailed information for all the actions performed in regards this agreement:

![userdoc.ip.accept.agreement.storage](assets/userdoc.ip.accept.agreement.storage.png)

Also we could see in the paymentHistory that the first prepay was already made and it will be available for the IP after the period ends.

### 2.5) Completing all the installment prepayments

In order to make this test simpler, we will just complete all the prepayments so the consumer does not get due with the payments. For doing this we just execute the same step as 2.3 the necessary times until all the installments are completed.

If we check the CHARLIE balance after this we will see the new reserved balance updated:

![userdoc.consumer.agreement.balance.allpaid](assets/userdoc.consumer.agreement.balance.allpaid.png)



### 2.6) IP withdraw and agreement completion.

As the agreemnt came to an end, the IP is able now to withdraw all the corresponding installments for the service. The IP can withdraw all at once by just executing the following extrinsic

![userdoc.ip.withdraw](assets/userdoc.ip.withdraw.png)

As a result, the following event will inform the change:

![userdoc.ip.withdraw.event](assets/userdoc.ip.withdraw.event.png)
