# 💸 Zeropolis – Smart Money Circles on Blockchain

Zeropolis is a decentralized ROSCA (Rotating Savings and Credit Association) platform that brings traditional money-saving circles onto the blockchain — making them **trustless**, **transparent**, and **fully programmable**.
```spl
Organizer
   │
   └──▶ create_cycle ─────┐
                          ▼
                    [Cycle Initialized]
                          │
         ┌────────────────┴────────────────┐
         ▼                                 ▼
  Members join_cycle                Organizer waits
         │                                 │
         └──▶ (members fill in)            │
                          │               ▼
                    [cycle.is_active ← true]
                          │
                    contribution rounds start
                          │
         ┌────────────────┴────────────────┐
         ▼                                 ▼
 submit_contribution             Organizer triggers_payout
         │                                 │
         └──▶ update_round, transfer       └──▶ payout & fee
                          │
     ┌──────────────┬──────────────┐
     ▼              ▼              ▼
 report_default  report_organizer_delay  exit_cycle
     │              │              │
     └───┐          └─────┐        └───┐
         ▼                ▼            ▼
  slash member      slash organizer   refund & leave
                          │
                          ▼
                [Cycle finishes all rounds]
                          │
                          ▼
                    close_cycle
                          │
                          ▼
              - Refund organizer stake
              - Refund/distribute remaining funds
              - Close accounts
```
---

## What’s the Idea?

Imagine a group of friends or community members saving together. Each month, everyone contributes a fixed amount, and one member takes the full pot. This continues until all members have received it once.

With **Zeropolis**, this entire process is automated by smart contracts — no banks, no middlemen, and no need to trust anyone.

---

## Key Features

- **Circle Creation**  
  Set group size, monthly contribution, payout rules (order/random/auction), and entry criteria.

- **Trustless Participation**  
  Members stake collateral, and smart contracts ensure timely payments and fair payouts.

- **Flexible Payout Logic**  
  Predefined, need-based, randomized, or auction-style distributions.

- **Dispute Handling**  
  Defaults are penalized via staked collateral. Members can vote on special cases.

- **On-chain Reputation**  
  Participants build a trust score based on their history across money circles.

- **Security Measures**  
  All funds are escrowed in the contract. Organizers can't claim the first payout.

---

## MVP Includes

- Web App: Create or join circles, track members and timelines.
- Smart Contract: Written in Anchor (Solana) to manage fund flows and logic.
- Reputation System: Public on-chain scores for accountability and access control.

---

## Why Zeropolis?

- Perfect for communities, families, and peer groups.
- Encourages savings without relying on financial institutions.
- Enforces rules through code — not trust.
