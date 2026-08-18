# EPIC-24: Demo Packaging

> **This EPIC lives in [pkdealer](https://github.com/ImperialBower/pkdealer).**
> Full design and implementation details:
> [`pkdealer/docs/EPIC-24_Demo.md`](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-24_Demo.md)

## Summary

Single-command demo launch of the full platform stack: `./demo.sh` starts
`docker-compose.yml` containing `pkdealer_service`, `pkdealer_spectator`,
bot agent containers (rule-based + Claude), Jaeger, Prometheus, Grafana, and
self-hosted Langfuse. Committed Grafana dashboard JSON and a `DEMO.md`
presenter guide complete the package.

**Status:** Complete (shipped in pkdealer — PR [#13](https://github.com/ImperialBower/pkdealer/pull/13))  
**Repo:** [ImperialBower/pkdealer](https://github.com/ImperialBower/pkdealer)  
**Depends on:** EPIC-20, EPIC-21, EPIC-22, EPIC-23  
**Runbook:** [`pkdealer/DEMO.md`](https://github.com/ImperialBower/pkdealer/blob/main/DEMO.md) · launcher [`demo.sh`](https://github.com/ImperialBower/pkdealer/blob/main/demo.sh) · five-agent stack in [`docker-compose.yml`](https://github.com/ImperialBower/pkdealer/blob/main/docker-compose.yml) (gto/lag/tag rules bots + random + ollama)
