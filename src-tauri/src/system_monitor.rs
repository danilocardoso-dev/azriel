use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemDetails {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub architecture: String,
    pub hostname: Option<String>,
    pub logical_cores: usize,
    pub physical_cores: Option<usize>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuSnapshot {
    pub usage_percent: f32,
    pub cores: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSnapshot {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub removable: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkSnapshot {
    pub interface_name: String,
    pub received_bytes: u64,
    pub transmitted_bytes: u64,
    pub received_bytes_total: u64,
    pub transmitted_bytes_total: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub collected_at: u64,
    pub details: SystemDetails,
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub storage: Vec<StorageSnapshot>,
    pub network: Vec<NetworkSnapshot>,
    pub errors: Vec<String>,
}

pub struct SystemMonitor {
    system: System,
    disks: Disks,
    networks: Networks,
}

impl SystemMonitor {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
            disks: Disks::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        }
    }

    pub fn snapshot(&mut self) -> SystemSnapshot {
        self.system.refresh_cpu_all();
        self.system.refresh_memory();
        self.disks.refresh(true);
        self.networks.refresh(true);
        let mut errors = Vec::new();
        let storage = self
            .disks
            .list()
            .iter()
            .map(|disk| StorageSnapshot {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: disk.file_system().to_string_lossy().to_string(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
                removable: disk.is_removable(),
            })
            .collect::<Vec<_>>();
        if storage.is_empty() {
            errors.push("Armazenamento indisponível".into());
        }
        let network = self
            .networks
            .iter()
            .map(|(name, data)| NetworkSnapshot {
                interface_name: name.clone(),
                received_bytes: data.received(),
                transmitted_bytes: data.transmitted(),
                received_bytes_total: data.total_received(),
                transmitted_bytes_total: data.total_transmitted(),
            })
            .collect::<Vec<_>>();
        if network.is_empty() {
            errors.push("Interfaces de rede indisponíveis".into());
        }
        SystemSnapshot {
            collected_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            details: SystemDetails {
                os_name: System::name(),
                os_version: System::os_version(),
                kernel_version: System::kernel_version(),
                architecture: System::cpu_arch(),
                hostname: System::host_name(),
                logical_cores: self.system.cpus().len(),
                physical_cores: System::physical_core_count(),
                uptime_seconds: System::uptime(),
            },
            cpu: CpuSnapshot {
                usage_percent: self.system.global_cpu_usage(),
                cores: self
                    .system
                    .cpus()
                    .iter()
                    .map(|cpu| cpu.cpu_usage())
                    .collect(),
            },
            memory: MemorySnapshot {
                total_bytes: self.system.total_memory(),
                used_bytes: self.system.used_memory(),
                available_bytes: self.system.available_memory(),
                swap_total_bytes: self.system.total_swap(),
                swap_used_bytes: self.system.used_swap(),
            },
            storage,
            network,
            errors,
        }
    }

    pub fn processes(&mut self) -> Vec<ProcessSnapshot> {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessSnapshot {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_percent: process.cpu_usage(),
                memory_bytes: process.memory(),
            })
            .collect()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SystemMonitorState(pub std::sync::Mutex<SystemMonitor>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_has_consistent_memory_and_system_fields() {
        let snapshot = SystemMonitor::new().snapshot();
        assert!(snapshot.memory.used_bytes <= snapshot.memory.total_bytes);
        assert!(!snapshot.details.architecture.is_empty());
        assert!(snapshot.cpu.usage_percent >= 0.0);
    }

    #[test]
    fn process_payload_excludes_sensitive_fields() {
        let serialized = serde_json::to_string(&SystemMonitor::new().processes()).unwrap();
        assert!(!serialized.contains("commandLine"));
        assert!(!serialized.contains("environment"));
        assert!(!serialized.contains("executablePath"));
    }
}
