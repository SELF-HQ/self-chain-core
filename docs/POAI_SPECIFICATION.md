# Proof-of-AI (PoAI) Consensus Mechanism

> **Source:** This specification is derived from the official PoAI whitepaper at [proofofai.com](https://proofofai.com).
>
> **Implementation Note:** This repository contains working implementations of core PoAI mechanisms including color markers, transaction selection (20/20/50/10), delegated keys, and voting. See `src/consensus/` for details.
>
> **AI Compute:** The AI validation component is configurable per constellation. Constellations choose their own AI infrastructure based on their requirements.

---

## Background
Blockchain technology is based on a distributed ledger system allowing secure, transparent, and tamper-proof transactions recorded as blocks. Each block contains a hash of the previous block, a timestamp, and transaction data. Hence, the blocks form a chain, as each block depends on the previous block.

One feature of blockchain technology is that no centralized authority determines the validity of transactions and adds them to the blockchain; instead, a consensus mechanism is required by which nodes in a blockchain network add blocks to the blockchain. The consensus mechanism should be resistant to spoofing, in which a bad actor seeks to add blocks containing fraudulent or inaccurate information.

In conventional blockchain technology, one type of consensus mechanism is Proof of Work (PoW) in which appending entities must expend significant computational effort to add blocks to the blockchain. One example of a PoW mechanism is the mining process, using, for example, Bitcoin, in which miners compete to mine tokens (e.g., cryptocurrency) and append blocks. By design, this type of consensus mechanism demands computational power and is, therefore, wasteful of energy.

Another type of conventional consensus mechanism is Proof of Stake (PoS), in which appending entities must hold a minimum number of blockchain tokens to add blocks to the blockchain. PoS is less energy-intensive than PoW but can lead to centralization in which a few entities holding many tokens come to control the network. PoS is also vulnerable to attack because of the low computational cost of spoofing.

Introducing Proof-of-AI (PoAI)
An alternative solution to common consensus mechanisms is a novel patent-pending consensus mechanism known as Proof-of-AI (PoAI), invented by Jonathan MacDonald. This document introduces the PoAI concept and describes the invention.

At its core, every part of PoAI drives efficiency by removing human interference and establishing the objective of the AI to fulfil its task in the most useful way. This is distinctly opposite to PoW (which is inefficient due to the mechanical computing involved) and PoS (which is increasingly centralized and therefore increasingly inefficient).

PoAI is based on the work of the following three algorithms that carry out their work independently, enabling blockchain consensus to be achieved.

The AI-Block Builder Algorithm: this forms effective blocks of transactions.
The Voting Algorithm: this is implemented by the PoAI mechanism, the task of which will be to organize voting for blocks of block builders and communicate between the two associated AI algorithms to achieve final consensus.
The AI-Validator Algorithm: this votes for the choice of block builder and determines the node with the permission to enter a block into the chain.

An overview of the PoAI architecture is shown here:

Proof of AI (PoAI) blockchain consensus mechanism
AI-Block Builder Algorithm
Block builders are full blockchain nodes that store ready-made blocks of the blockchain and participate in forming and packaging new blocks. The block builder's basis is a Machine Learning (ML) model, which has the initial, specified parameters of the methodology for forming effective blocks from existing mempool transactions.

The model can also learn and improve the efficiency of its work in selecting transactions. The block builder considers transactions from the mempool for the block in the following priority (n - number of transactions in the block):

Highest price (>0.2n transactions)
Lowest price (>0.2n transactions) - to compensate highest prices
Average price (>0.5n)
Oldest transactions (>0.1n)

One essential aspect of PoAI is making transactions optimized for affordability. This is why the block builder packs the blocks as much as possible and why the blockchain includes a halving algorithm that drives the reliance on transaction efficiency. Due to the dynamic nature of PoAI, the mechanism can sense and respond to attack vectors and attempts at manipulation and adjust accordingly to mitigate threats.

Each block requires a certain volume of Points to be created (this is known as the PointPrice). The goal is to align the number of points for each block with the number of coins generated. For example, 1 point equals 0.001 coins. If the block requires a PointPrice of 10,000, the blockchain generates 10 new coins. When the blockchain reaches a certain amount of total PointPrice spent (for example, 30,000,000,000 points), each point becomes equal to 0.0005 coins. After reaching the second milestone (for example, 60,000,000,000), each point equals 0.00025 coins.

An effective block is any block assembled so that the average PointPrice is close to the actual PointPrice. It includes transactions with the maximum useful information that can be entered into the block. For clarification, ‘useful informationʼ is PointData (the volume of Points). The goal is for the maximum amount of PointData to be included. A limited number of transactions with a higher-than-average PointPrice should reduce the number of users in the system who set a high PointPrice but still favor and compensate them with a limited number of transactions with a low PointPrice. Most of the average PointPrice transactions will be validated, incentivising users not to set high PointPrices.

In contrast, other networks choose the highest gas fees, leading to a negative user experience. The alternative version provided by PoAI is illustrated below, where the vertical axis is the PointPrice, and the horizontal is the data length. The center of the target is the chosen price by the algorithm:

Proof of AI (PoAI) blockchain consensus mechanism
The effectiveness of the ML model depends on the server's technical resources and the frequency of block assembly. A block builder's ML model is trained based on its work in selecting transactions for a block, comparing its results with those of other block builders, and the final voting result, which determines whose block will be sent to the chain.
‍
The more often a block builder forms a block, and the more mempool transactions it manages to sort through during the assembly of a new block, the more efficient the block is and the greater the chance that this particular block will be chosen for inclusion in the chain at the next stage. After the generation of the current block is completed and entered into the chain, a timer is started to assemble a new block. During the timer, all block builders are sorting through mempool transactions and selecting the best transactions to form a block. At the end of the timer, all block builders put their blocks forward to a voting process. The voting application includes the following information:

Block assembly
Block efficiency (% filling of the block with useful information)
Income of the block builder during its existence
Timestamp of the last block assembly
Percentage of votes out of the total number of attempts
Percentage of victory in voting out of the total number of people going to vote
The below figure illustrates how PoAI selects which block builders can participate in the round:

Proof of AI (PoAI) blockchain consensus mechanism
Voting Algorithm
The PoAI mechanism organises voting and cannot directly influence its results or participate in voting. It will have an identical ML model internally to that of the block builder and generate its own version of the block with each new round of assembly.

PoAI and block builders review transactions from the mempool and build an efficient block for the round. The block generated by PoAI is considered as the reference block and is compared with the blocks generated by the builders. If the newly generated block is more efficient than the current reference block, it becomes the new reference block.

Proof of AI (PoAI) blockchain consensus mechanism
PoAI removes block builders who have successfully submitted blocks to the chain during the last N-block period from the voting list. Implementing such a timeout allows all block builders to receive a reward for assembling a block and reduces the risk of unfair assemblies from one of the participants. The PoAI mechanism participates in all block assemblies from the very beginning of the chain. It will also have information about all voting results, and the block collected by the PoAI mechanism will be considered the reference block.

The chosen reference block is vital and ensures that, despite users' choices, manipulating the mechanism is improbable, as it removes manual interference. For each block builder admitted to voting, the PoAI mechanism will check the accuracy of the efficiency coefficient based on the percentage of the block filled with useful information, assess its quality, comparing it with its reference block.

The coefficient is the numeric value based on the efficiency (Input minus Output) of the amount of block creation. So, two objectives are required:

The volume of Points in the block is biased towards being the maximum possible.
The PointPrice is as stable as possible, so it is biased toward being as similar as possible to the previous block generation.
If the useful efficiency coefficient is higher than that of the reference block, the PoAI mechanism will give such a block builder the highest score and, after double-checking, consider its block the reference block for the current round of assembly. Block quality is an additional parameter, not the main one for voting, but one that the AI-validator model algorithm can use when making a decision.

AI-Validator Algorithm
AI validators are lite blockchain nodes that store network wallet addresses and color markers. Color markers are a key part of validation and connecting a block to the blockchain:

Each wallet in the system is classed as a HEX wallet as it has a colour attributed to it using a hexadecimal value. When a user signs a transaction in their crypto-wallet, a hexidecimal hash is generated.
The AI validator stores information about the current color of the wallet.
The block builder divides the transaction hash into six parts. It divides each part into two and adds them until it gets a number of one character. Thus, it receives six numbers in HEX, which the block builder glues into a single number, which we term a HEX transaction. Next, the block builder adds the HEX wallet with the HEX transaction and receives a new HEX wallet.
The HEX of the new HEX wallet and the corresponding transaction hash are sent to the block for which AI validators are selected. AI validators are selected randomly among all those who voted for the AI builder in this round but only to those whose votes were not given to the winning AI builder (the random number is selected according to a special formula). AI validators also form the transaction hash into a transaction HEX and add it to the existing HEX wallet, obtaining a final HEX wallet.
If the new HEX calculated by the AI validator coincides with the new HEX of the wallet transmitted in the block for all wallets, then the AI validator considers the block valid.
The colors of the wallets change to those obtained by adding their current colors with HEX transactions.
Lite nodes can also send and receive transactions to power the blockchain wallet application. For a wallet address to become an AI validator, it must satisfy the following conditions:

In the last N hours there have been transactions at the wallet address (I.E., the wallet is active);
the wallet address contains the N-amount of the native currency of the blockchain network.
The figure below illustrates how PoAI determines which validators are eligible to vote in the round:

Proof of AI (PoAI) blockchain consensus mechanism
Validation Process
AI validators participate in every vote to determine the block builder with each new round of block assembly. The AI validator votes independently, without user intervention, in the background. The AI validator's artificial intelligence model is being trained based on the results of previous voting and analysis of the participants' block builders' parameters, as well as the requested manual votes of the AI validator's owner.

Once the AI validator has voted, it requests user participation in voting at the beginning of its network activity and, using a special algorithm, reduces the number of requests to the wallet owner over time.
Human participation involves choosing one of two block builder options the AI validator provides. Every user has a different choice of two, and the choices become increasingly shortlisted. The result of the userʼs (wallet ownerʼs) selection is added to the validatorʼs knowledge base and used in the further operation of the AI model.
In the subsequent period, voting occurs automatically. At a certain point, the AI validator will again send requests for manual voting to replenish its knowledge base and verify that the wallet's owner is a real network user and is active.
If a wallet owner does not participate in several manual votes in a row, the associated AI validator is questioned, and the AI validator's votes may not be counted in subsequent rounds. This penalty is to encourage ongoing participation in securing the network.
The order and timing of requests for manual voting are determined randomly to prevent collective voting. An AI validator's owner cannot independently request the possibility of manual voting or influence the timing of its proposal.
Validators admitted to the voting process select the block for the round as illustrated below:

Proof of AI (PoAI) blockchain consensus mechanism
After the voting is completed and the winning block builder is determined, the PoAI mechanism selects one from the active AI validators. The validators additionally double-check the collected block before placing it on the blockchain. The AI validator for color-marker validation of block transactions is selected according to the following parameters:

The AI validator is active and valid according to the conditions listed above.
The AI validator did not vote for the winning block builder.
Among the validators who voted, one validator is randomly selected from those who did not vote for the block that won the round. This validator will use color validation to check all transactions in the block.

Proof of AI (PoAI) blockchain consensus mechanism
Based on the formula for calculating the random value, the serial number of the AI validator is determined from the list of voters, which will perform color-marker validation of block transactions. The following image illustrates these concepts.

Proof of AI (PoAI) blockchain consensus mechanism
The Incentive
The reward for participation is generated as a native coin, earned by helping to assemble a new block among participants in the consensus process. An example reward distribution is as follows:

90% goes to the block builder;
8% is distributed among the AI validators who voted for the winning block builder;
1% goes to the AI validator, who double-checked the block (according to the color scheme);
1% of the reward goes to PoAI mechanisms (credited to the blockchain's reserve fund) for organizing the voting procedure.

These percentages can be revised to reflect each consensus participant's labor costs and contribution to its implementation. The example above is illustrated below:

---

## Implementation Notes for SELF Chain

**Production Implementation (SELF App, January 2026):**

This repository implements the core PoAI consensus mechanisms as specified above. However, **SELF Chain constellations can customize reward mechanisms** to fit their use case.

**SELF App (First Constellation) uses a Prize Pool System:**
- Each validator vote creates a prize draw entry (1 vote = 1 entry)
- Winners are selected via daily/weekly/monthly prize drawings (5k/50k/200k SELF)
- No per-round blockchain rewards are distributed
- This provides better incentives and user experience compared to negligible per-round rewards
- Prize draws are cryptographically verifiable (SHA256-based selection)

**Default PoAI Distribution (90/8/1/1) is also available:**
- 90% to block builder
- 8% split among voters who voted for winning block
- 1% to color marker validator
- 1% to network reserve

Constellations implement their own reward mechanisms via the `RewardDistributor` trait (see `examples/custom_rewards.rs`).

**Browser-Based Validators:**
- SELF Chain implements browser-based validators (not server-based)
- Validator keys derived from recovery phrase (BIP32 `m/44'/60'/1'/0/0`)
- Keys held in browser memory only (never transmitted)
- WebSocket connection to coordinator for voting
- Zero-knowledge architecture preserves user sovereignty

**Production Status:**
- Live since January 1, 2026
- Coordinator: ``
- 60-second consensus rounds
- Browser-based validators with real users

---

## Implementation Notes for SELF Chain

**Production Implementation (SELF App, January 2026):**

This repository implements the core PoAI consensus mechanisms as specified above. However, **SELF Chain constellations can customize reward mechanisms** to fit their use case.

**SELF App (First Constellation) uses a Prize Pool System:**
- Each validator vote creates a prize draw entry (1 vote = 1 entry)
- Winners are selected via daily/weekly/monthly prize drawings (5k/50k/200k SELF)
- No per-round blockchain rewards are distributed
- This provides better incentives and user experience compared to negligible per-round rewards
- Prize draws are cryptographically verifiable (SHA256-based selection)

**Default PoAI Distribution (90/8/1/1) is also available:**
- 90% to block builder
- 8% split among voters who voted for winning block
- 1% to color marker validator
- 1% to network reserve

Constellations implement their own reward mechanisms via the `RewardDistributor` trait (see `examples/custom_rewards.rs`).

**Browser-Based Validators:**
- SELF Chain implements browser-based validators (not server-based)
- Validator keys derived from recovery phrase (BIP32 `m/44'/60'/1'/0/0`)
- Keys held in browser memory only (never transmitted)
- WebSocket connection to coordinator for voting
- Zero-knowledge architecture preserves user sovereignty

**Production Status:**
- Live since January 1, 2026
- Coordinator: ``
- 60-second consensus rounds
- Browser-based validators with real users
