# 💸 Zeropolis – Smart Money Circles on Blockchain

Zeropolis is a decentralized ROSCA (Rotating Savings and Credit Association) platform that brings traditional money-saving circles onto the blockchain — making them **trustless**, **transparent**, and **fully programmable**.

![Solana](https://img.shields.io/badge/Solana-Devnet-3ECF8E?logo=solana&logoColor=white)
![Anchor](https://img.shields.io/badge/Anchor-Framework-blueviolet)

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

---


---
```scharp
Organizer
   │
   └──▶ create_cycle(target_participants, collateral, token_mint, ...)
          │
          ▼
    [Cycle PDA account is initialized]
          │
          ▼
    [Cycle token_account (ATA) is created]
          │
          ▼
    [cycle.current_participants = 0]
          │
          ▼
Participants start joining
          │
          └──▶ join_cycle()
                   │
                   ▼
         [member_account created with is_active = true]
                   │
                   ▼
         [collateral transferred to cycle_token_account]
                   │
                   ▼
         [member added to payout_order]
                   │
                   ▼
         [cycle.current_participants += 1]
                   │
                   ▼
         ┌────────────── if ───────────────┐
         │ cycle.current_participants ==   │
         │        cycle.target_participants│
         └─────────────────┬───────────────┘
                           │
                           ▼
               [Cycle starts automatically]
                           │
                           ▼
              [cycle.current_round = 1]
                           │
                           ▼
           ┌─────────────────────────────────┐
           │  For each round in the cycle:   │
           └─────────────────────────────────┘
                           │
         ┌─────────────────┴──────────────────┐
         ▼                                    ▼
members submit_contribution()         (no more exits allowed)
         │
         ▼
[Pot collected for the round]
         │
         ▼
[payout_order[current_round - 1] gets the pot]
         │
         ▼
[cycle.current_round += 1]
         │
   ┌─────┴────────────┐
   ▼                  ▼
 more rounds        all rounds done
    left              │
    │                 ▼
    └────────────▶ [Cycle closes]
```
