# EPIC-20: Autonomous Game Loop

> **This EPIC lives in [pkdealer](https://github.com/ImperialBower/pkdealer).**
> Full design and implementation details:
> [`EPIC-20_Autonomous_Game_Loop.md`](https://EPIC-20_Autonomous_Game_Loop.md)

## Summary

Migrate `pkdealer_service` from `pkcore::Dealer` to `pkcore::PokerSession`,
removing the `unsafe impl Send` workaround and enabling autonomous street
advancement — the service auto-advances streets and ends hands after all
players have acted, without requiring explicit `AdvanceStreet` / `EndHand`
RPC calls from clients.

**Status:** Planned  
**Repo:** [ImperialBower/pkdealer](https://github.com/ImperialBower/pkdealer)  
**pkcore dependency:** `PokerSession` (`src/casino/session.rs`), `PlayerAction` (`src/bot/player_action.rs`)
