# Operations History

## 2026-06-27 07:38:22 — Codex
Moved nova monitoring discovery task directory from `ops/in_progress/` to `ops/completed/`.

Files touched:
- ops/in_progress/nova-monitoring-discovery/
- ops/completed/nova-monitoring-discovery/
- ops/operations_history.md
- ops/decisions.log

— Codex

## 2026-06-27 07:37:58 — Codex
Built release binary, installed and started `agentsight-monitor.service`, verified active nova discovery from the running service, and wrote completion report.

Files touched:
- collector/target/release/agentsight
- /home/x/.config/systemd/user/agentsight-monitor.service
- /home/x/.agentsight/monitor/monitor-2026-W26.db
- ops/in_progress/nova-monitoring-discovery/completion_report.md
- ops/operations_history.md
- ops/decisions.log

— Codex

## 2026-06-27 07:35:49 — Codex
Implemented nova systemd discovery in the AgentSight background monitor and verified it against the live host.

Files touched:
- collector/src/sources/nova.rs
- collector/src/sources/mod.rs
- collector/src/cmd_monitor.rs
- collector/src/output/format.rs
- collector/src/output/tui.rs
- docs/usage.md
- ops/operations_history.md
- ops/decisions.log

Receipts:
- `systemctl list-units 'nova@nova-*.service' --type=service --no-legend --no-pager --plain` returned `nova@nova-001.service` through `nova@nova-024.service`, all loaded active running.
- `/var/lib/novacol/novas/nova-001`, `nova-002`, and `nova-024` homes, identity files, and env files exist; `/opt/novacol/bin/nova-core` is executable.
- `curl --max-time 5 http://127.0.0.1:8765/api/status` returned HTTP 200 JSON with `service_summary.running_novas: 24`.
- NATS on `127.0.0.1:4222` returned an INFO banner for `novacol-nats`.
- `./collector/target/debug/agentsight top --plain --once -n 40` against a live monitor showed nova rows for `nova-001`, `nova-002`, and `nova-003` with systemd evidence.
- System Python SQLite inspection of `/home/x/.agentsight/monitor/monitor-2026-W26.db` showed 24 distinct nova sessions with `match_evidence=systemd`.
- `cargo test --manifest-path collector/Cargo.toml` passed: 135 unit tests, 5 export snapshot tests, and 3 system runner tests; 5 live/sudo smoke tests ignored by design.

— Codex

## 2026-06-27 07:27:51 — Codex
Moved from planning into implementation under the active nova monitoring continuation request.

Files touched:
- plans/nova-monitoring-discovery.md
- ops/operations_history.md
- ops/decisions.log

— Codex

## 2026-06-27 07:27:31 — Codex
Added `.gitignore` exception so required operational decisions log is tracked despite the repository-wide `*.log` ignore rule.

Files touched:
- .gitignore
- ops/operations_history.md
- ops/decisions.log

— Codex

## 2026-06-27 07:27:08 — Codex
Commit attempt failed because Git author identity was unset; configured repository-local `user.name=Codex` and `user.email=codex@adapt.local`, then prepared to retry the required commit.

Files touched:
- .git/config
- ops/operations_history.md
- ops/decisions.log

— Codex

## 2026-06-27 07:26:27 — Codex
Created operational task scaffold for nova monitoring discovery continuation.

Files touched:
- ops/to_do/
- ops/in_progress/nova-monitoring-discovery/
- ops/completed/
- plans/nova-monitoring-discovery.md

— Codex
