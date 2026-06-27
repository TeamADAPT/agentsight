# Nova Monitoring Discovery Task

## 2026-06-27 07:26:27 — Codex
Objective: install and configure AgentSight monitoring so it discovers nova agents from live systemd units matching `nova@nova-*.service`, maps `%i` to `/var/lib/novacol/novas/%i`, reads identity/env context, and exposes the discovered agents through the existing monitor/top path.

Required live sources:
- Systemd units: `nova@nova-001.service` through `nova@nova-024.service`
- Runtime homes: `/var/lib/novacol/novas/nova-001` through `/var/lib/novacol/novas/nova-024`
- Identity files: `/var/lib/novacol/novas/nova-###/config/identity.toml`
- Env files: `/etc/novacol/nova-nova-###.env`
- Main binary: `/opt/novacol/bin/nova-core`
- Dashboard API: `http://127.0.0.1:8765/api/status`
- NATS URL: `nats://127.0.0.1:4222`
- NATS subjects: `nova.>`, `nova.nova-###.inbox`, `nova.nova-###.chatter`

Source of truth: `systemctl list-units 'nova@nova-*.service' --type=service`, then map `%i` to `/var/lib/novacol/novas/%i`.

— Codex
