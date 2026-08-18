# EPIC-22: OpenTelemetry Instrumentation

> **This EPIC lives in [pkdealer](https://github.com/ImperialBower/pkdealer).**
> Full design and implementation details:
> [`EPIC-22_OTel.md`](https://EPIC-22_OTel.md)

## Summary

Instrument `pkdealer_service` with OpenTelemetry: `hand`, `street`, and
`action` spans exported via OTLP; `pkdealer.hands_played`, `pkdealer.pot_size`,
and `pkdealer.action_duration_ms` metrics. Trace context is propagated through
gRPC metadata so agent decision spans nest under service action spans. A
`docker-compose.yml` ships Jaeger, Prometheus, and Grafana.

**Status:** Planned  
**Repo:** [ImperialBower/pkdealer](https://github.com/ImperialBower/pkdealer)  
**Depends on:** EPIC-20 (Autonomous Game Loop)
