# EPIC-21: Web Spectator App

> **This EPIC lives in [pkdealer](https://github.com/ImperialBower/pkdealer).**
> Full design and implementation details:
> [`EPIC-21_Spectator.md`](https://EPIC-21_Spectator.md)

## Summary

Add a new `pkdealer_spectator` crate to the pkdealer workspace — an Axum web
server that subscribes to `pkdealer_service`'s `StreamEvents` RPC (using the
spectator token) and re-broadcasts table events to browsers over SSE. The
frontend renders all hole cards, the board, pot, chip counts, and an action
log in real time.

**Status:** Planned  
**Repo:** [ImperialBower/pkdealer](https://github.com/ImperialBower/pkdealer)  
**Depends on:** EPIC-20 (Autonomous Game Loop)
