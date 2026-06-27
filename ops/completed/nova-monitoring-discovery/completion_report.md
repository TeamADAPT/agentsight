# Nova Monitoring Discovery Completion Report

## 2026-06-27 07:37:58 — Codex
Completed implementation, installation, and live verification for nova discovery in AgentSight background monitoring.

Changes delivered:
- Added `collector/src/sources/nova.rs` to discover nova instances from `systemctl list-units 'nova@nova-*.service' --type=service`.
- Mapped each systemd `%i` to `/var/lib/novacol/novas/%i`, `/var/lib/novacol/novas/%i/config/identity.toml`, `/etc/novacol/nova-%i.env`, `/opt/novacol/bin/nova-core`, dashboard API `http://127.0.0.1:8765/api/status`, NATS URL `nats://127.0.0.1:4222`, and subjects `nova.>`, `nova.%i.inbox`, `nova.%i.chatter`.
- Wired nova instances into `collector/src/cmd_monitor.rs` so active `MainPID` values become process-family roots and inactive loaded units remain visible as systemd DB rows.
- Added first-class `systemd` evidence labels in monitor/top output.
- Documented nova monitor discovery in `docs/usage.md`.
- Built release binary and installed/enabled/restarted `agentsight-monitor.service` through systemd user service management.

Verification receipts:
- `cargo test --manifest-path collector/Cargo.toml` passed: 135 unit tests, 5 export snapshot tests, 3 system runner tests; live/sudo smoke tests remained ignored by design.
- `systemctl list-units 'nova@nova-*.service' --type=service --no-legend --no-pager --plain` returned `nova@nova-001.service` through `nova@nova-024.service`, all loaded active running.
- `systemctl show` confirmed `nova@nova-001.service` `MainPID=911019` and `nova@nova-024.service` `MainPID=911214`.
- Runtime homes, identity files, and env files exist for sampled `nova-001`, `nova-002`, and `nova-024`; `/opt/novacol/bin/nova-core` is executable.
- `curl --max-time 5 http://127.0.0.1:8765/api/status` returned HTTP 200 JSON with `service_summary.running_novas: 24`.
- NATS `127.0.0.1:4222` returned an INFO banner for `novacol-nats`.
- `systemctl --user status agentsight-monitor.service --no-pager` showed the release monitor active, enabled, and writing `/home/x/.agentsight/monitor/monitor-2026-W26.db`.
- `./collector/target/release/agentsight top --plain --once -n 40` showed nova monitor rows for `nova-001`, `nova-002`, and `nova-003`.
- System Python SQLite inspection of `/home/x/.agentsight/monitor/monitor-2026-W26.db` showed 24 distinct nova sessions with `match_evidence=systemd`, including `nova-001|911019`, `nova-002|911023`, and `nova-003|911025`.

— Codex
