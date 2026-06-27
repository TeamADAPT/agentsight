// SPDX-License-Identifier: MIT
// Copyright (c) 2026 eunomia-bpf org.

use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub(crate) const NOVA_UNIT_PATTERN: &str = "nova@nova-*.service";
pub(crate) const NOVA_HOME_ROOT: &str = "/var/lib/novacol/novas";
pub(crate) const NOVA_ENV_ROOT: &str = "/etc/novacol";
pub(crate) const NOVA_MAIN_BINARY: &str = "/opt/novacol/bin/nova-core";
pub(crate) const NOVA_DASHBOARD_API: &str = "http://127.0.0.1:8765/api/status";
pub(crate) const NOVA_NATS_URL: &str = "nats://127.0.0.1:4222";
pub(crate) const NOVA_NATS_SUBJECT_ROOT: &str = "nova.>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NovaSystemdUnit {
    pub(crate) unit: String,
    pub(crate) instance: String,
    pub(crate) load_state: Option<String>,
    pub(crate) active_state: Option<String>,
    pub(crate) sub_state: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NovaSystemdState {
    pub(crate) unit: String,
    pub(crate) main_pid: Option<u32>,
    pub(crate) active_state: Option<String>,
    pub(crate) sub_state: Option<String>,
    pub(crate) load_state: Option<String>,
    pub(crate) fragment_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NovaInstance {
    pub(crate) unit: String,
    pub(crate) instance: String,
    pub(crate) home: PathBuf,
    pub(crate) identity_path: PathBuf,
    pub(crate) env_path: PathBuf,
    pub(crate) binary_path: PathBuf,
    pub(crate) dashboard_api: String,
    pub(crate) nats_url: String,
    pub(crate) nats_subject_root: String,
    pub(crate) inbox_subject: String,
    pub(crate) chatter_subject: String,
    pub(crate) main_pid: Option<u32>,
    pub(crate) active_state: Option<String>,
    pub(crate) sub_state: Option<String>,
    pub(crate) load_state: Option<String>,
    pub(crate) fragment_path: Option<String>,
}

impl NovaInstance {
    pub(crate) fn command_summary(&self) -> String {
        let pid = self
            .main_pid
            .map(|pid| pid.to_string())
            .unwrap_or_else(|| "none".to_string());
        let active = self.active_state.as_deref().unwrap_or("unknown");
        let sub = self.sub_state.as_deref().unwrap_or("unknown");
        format!(
            "{} unit={} pid={} state={}/{} home={} identity={} env={} dashboard={} nats={} inbox={} chatter={}",
            self.binary_path.display(),
            self.unit,
            pid,
            active,
            sub,
            self.home.display(),
            self.identity_path.display(),
            self.env_path.display(),
            self.dashboard_api,
            self.nats_url,
            self.inbox_subject,
            self.chatter_subject
        )
    }

    pub(crate) fn config_paths(&self) -> Vec<PathBuf> {
        vec![
            self.home.clone(),
            self.identity_path.clone(),
            self.env_path.clone(),
            self.binary_path.clone(),
        ]
    }
}

pub(crate) fn discover() -> io::Result<Vec<NovaInstance>> {
    let units = list_systemd_units()?;
    let unit_names = units
        .iter()
        .map(|unit| unit.unit.as_str())
        .collect::<Vec<_>>();
    let states = match show_systemd_states(&unit_names) {
        Ok(states) => states,
        Err(err) => {
            log::warn!("failed to read nova systemd MainPID state: {}", err);
            BTreeMap::new()
        }
    };
    Ok(build_instances(&units, &states))
}

fn list_systemd_units() -> io::Result<Vec<NovaSystemdUnit>> {
    let output = Command::new("systemctl")
        .args([
            "list-units",
            NOVA_UNIT_PATTERN,
            "--type=service",
            "--no-legend",
            "--no-pager",
            "--plain",
        ])
        .output()?;
    if !output.status.success() {
        return Err(systemctl_error("systemctl list-units", &output));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_systemctl_list_units(&stdout))
}

fn show_systemd_states(unit_names: &[&str]) -> io::Result<BTreeMap<String, NovaSystemdState>> {
    if unit_names.is_empty() {
        return Ok(BTreeMap::new());
    }

    let output = Command::new("systemctl")
        .arg("show")
        .arg("--no-pager")
        .arg("--property=Id")
        .arg("--property=MainPID")
        .arg("--property=ActiveState")
        .arg("--property=SubState")
        .arg("--property=LoadState")
        .arg("--property=FragmentPath")
        .args(unit_names)
        .output()?;
    if !output.status.success() {
        return Err(systemctl_error("systemctl show", &output));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_systemctl_show_units(&stdout))
}

fn systemctl_error(command: &str, output: &std::process::Output) -> io::Error {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    io::Error::other(format!(
        "{command} failed with status {}: {}{}",
        output.status, stdout, stderr
    ))
}

pub(crate) fn parse_systemctl_list_units(text: &str) -> Vec<NovaSystemdUnit> {
    let mut out = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let mut fields = line.split_whitespace();
        let Some(raw_unit) = fields.next() else {
            continue;
        };
        let unit = normalize_unit_token(raw_unit);
        let Some(instance) = parse_nova_unit_instance(&unit) else {
            continue;
        };
        out.push(NovaSystemdUnit {
            unit,
            instance,
            load_state: fields.next().map(ToString::to_string),
            active_state: fields.next().map(ToString::to_string),
            sub_state: fields.next().map(ToString::to_string),
        });
    }
    out
}

fn normalize_unit_token(value: &str) -> String {
    value
        .trim_start_matches(|ch| ch == '\u{25cf}' || ch == '*')
        .trim()
        .to_string()
}

pub(crate) fn parse_nova_unit_instance(unit: &str) -> Option<String> {
    let instance = unit.strip_prefix("nova@")?.strip_suffix(".service")?;
    if instance.starts_with("nova-") && instance.len() > "nova-".len() {
        Some(instance.to_string())
    } else {
        None
    }
}

pub(crate) fn parse_systemctl_show_units(text: &str) -> BTreeMap<String, NovaSystemdState> {
    let mut out = BTreeMap::new();
    let mut current = NovaSystemdState::default();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            flush_state(&mut out, &mut current);
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "Id" => current.unit = value.to_string(),
            "MainPID" => {
                current.main_pid = value
                    .parse::<u32>()
                    .ok()
                    .and_then(|pid| (pid > 0).then_some(pid));
            }
            "ActiveState" => current.active_state = non_empty_value(value),
            "SubState" => current.sub_state = non_empty_value(value),
            "LoadState" => current.load_state = non_empty_value(value),
            "FragmentPath" => current.fragment_path = non_empty_value(value),
            _ => {}
        }
    }
    flush_state(&mut out, &mut current);
    out
}

fn flush_state(out: &mut BTreeMap<String, NovaSystemdState>, current: &mut NovaSystemdState) {
    if !current.unit.is_empty() {
        out.insert(current.unit.clone(), std::mem::take(current));
    }
}

fn non_empty_value(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

pub(crate) fn build_instances(
    units: &[NovaSystemdUnit],
    states: &BTreeMap<String, NovaSystemdState>,
) -> Vec<NovaInstance> {
    units
        .iter()
        .map(|unit| {
            let paths = NovaInstancePaths::for_instance(&unit.instance);
            let state = states.get(&unit.unit);
            NovaInstance {
                unit: unit.unit.clone(),
                instance: unit.instance.clone(),
                home: paths.home,
                identity_path: paths.identity_path,
                env_path: paths.env_path,
                binary_path: PathBuf::from(NOVA_MAIN_BINARY),
                dashboard_api: NOVA_DASHBOARD_API.to_string(),
                nats_url: NOVA_NATS_URL.to_string(),
                nats_subject_root: NOVA_NATS_SUBJECT_ROOT.to_string(),
                inbox_subject: format!("nova.{}.inbox", unit.instance),
                chatter_subject: format!("nova.{}.chatter", unit.instance),
                main_pid: state.and_then(|state| state.main_pid),
                active_state: state
                    .and_then(|state| state.active_state.clone())
                    .or_else(|| unit.active_state.clone()),
                sub_state: state
                    .and_then(|state| state.sub_state.clone())
                    .or_else(|| unit.sub_state.clone()),
                load_state: state
                    .and_then(|state| state.load_state.clone())
                    .or_else(|| unit.load_state.clone()),
                fragment_path: state.and_then(|state| state.fragment_path.clone()),
            }
        })
        .collect()
}

struct NovaInstancePaths {
    home: PathBuf,
    identity_path: PathBuf,
    env_path: PathBuf,
}

impl NovaInstancePaths {
    fn for_instance(instance: &str) -> Self {
        let home = PathBuf::from(NOVA_HOME_ROOT).join(instance);
        let identity_path = home.join("config").join("identity.toml");
        let env_path = PathBuf::from(NOVA_ENV_ROOT).join(format!("nova-{instance}.env"));
        Self {
            home,
            identity_path,
            env_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nova_systemd_unit_list() {
        let units = parse_systemctl_list_units(
            "\
nova@nova-001.service loaded active running Nova worker 001
nova@nova-024.service loaded activating start Nova worker 024
ssh.service loaded active running SSH daemon
",
        );

        assert_eq!(units.len(), 2);
        assert_eq!(units[0].unit, "nova@nova-001.service");
        assert_eq!(units[0].instance, "nova-001");
        assert_eq!(units[0].active_state.as_deref(), Some("active"));
        assert_eq!(units[1].instance, "nova-024");
        assert_eq!(units[1].sub_state.as_deref(), Some("start"));
    }

    #[test]
    fn parses_systemctl_show_blocks() {
        let states = parse_systemctl_show_units(
            "\
Id=nova@nova-001.service
MainPID=101
ActiveState=active
SubState=running
LoadState=loaded
FragmentPath=/etc/systemd/system/nova@.service

Id=nova@nova-002.service
MainPID=0
ActiveState=failed
SubState=failed
LoadState=loaded
FragmentPath=/etc/systemd/system/nova@.service
",
        );

        assert_eq!(
            states
                .get("nova@nova-001.service")
                .and_then(|state| state.main_pid),
            Some(101)
        );
        assert_eq!(
            states
                .get("nova@nova-002.service")
                .and_then(|state| state.main_pid),
            None
        );
        assert_eq!(
            states
                .get("nova@nova-002.service")
                .and_then(|state| state.active_state.as_deref()),
            Some("failed")
        );
    }

    #[test]
    fn maps_instance_to_novacol_paths_and_subjects() {
        let units = vec![NovaSystemdUnit {
            unit: "nova@nova-007.service".to_string(),
            instance: "nova-007".to_string(),
            load_state: Some("loaded".to_string()),
            active_state: Some("active".to_string()),
            sub_state: Some("running".to_string()),
        }];
        let instances = build_instances(&units, &BTreeMap::new());
        let nova = &instances[0];

        assert_eq!(nova.home, PathBuf::from("/var/lib/novacol/novas/nova-007"));
        assert_eq!(
            nova.identity_path,
            PathBuf::from("/var/lib/novacol/novas/nova-007/config/identity.toml")
        );
        assert_eq!(
            nova.env_path,
            PathBuf::from("/etc/novacol/nova-nova-007.env")
        );
        assert_eq!(
            nova.binary_path,
            PathBuf::from("/opt/novacol/bin/nova-core")
        );
        assert_eq!(nova.dashboard_api, "http://127.0.0.1:8765/api/status");
        assert_eq!(nova.nats_url, "nats://127.0.0.1:4222");
        assert_eq!(nova.nats_subject_root, "nova.>");
        assert_eq!(nova.inbox_subject, "nova.nova-007.inbox");
        assert_eq!(nova.chatter_subject, "nova.nova-007.chatter");
    }
}
