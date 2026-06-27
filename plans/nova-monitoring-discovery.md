# Nova Monitoring Discovery Plan

## 2026-06-27 07:26:27 — Codex
Status: approved for implementation by active continuation request.

Implement nova discovery inside the existing AgentSight monitor path, keeping `collector/src/cmd_monitor.rs` as the primary integration point unless code inspection proves a smaller dedicated module is warranted.

Plan:
1. Add a nova discovery source that shells out to `systemctl list-units 'nova@nova-*.service' --type=service` and parses unit names as the authoritative instance list.
2. Map each `%i` instance to `/var/lib/novacol/novas/%i`, `/var/lib/novacol/novas/%i/config/identity.toml`, `/etc/novacol/nova-%i.env`, `/opt/novacol/bin/nova-core`, dashboard API `http://127.0.0.1:8765/api/status`, and NATS subjects `nova.%i.inbox` / `nova.%i.chatter`.
3. Resolve each systemd unit's live `MainPID` through `systemctl show` and use `/proc` to collect process family, CPU, RSS, file targets, and network targets.
4. Feed discovered nova instances into the monitor sample/store path as monitor sessions with stable IDs, display IDs, command/cwd fields, and evidence marked as systemd.
5. Add focused unit tests for systemd unit parsing, instance-to-path mapping, and monitor sample conversion without requiring systemd.
6. Verify with Rust tests, then perform live receipt checks against `systemctl`, filesystem paths, the dashboard API, and NATS reachability when those services exist on the host.

Files expected to change:
- collector/src/cmd_monitor.rs
- collector/src/main.rs only if CLI/help text needs adjustment
- docs/usage.md or focused docs only if monitor behavior becomes user-facing
- ops/operations_history.md
- ops/decisions.log
- ops/in_progress/nova-monitoring-discovery/completion_report.md on completion

— Codex
