use anyhow::Result;
use serde::{Deserialize, Serialize};
use sysinfo::Disks;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub mount_point: String,
    pub fs_type: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub is_removable: bool,
    pub is_read_only: bool,
}

pub fn list_volumes() -> Result<Vec<VolumeInfo>> {
    let disks = Disks::new_with_refreshed_list();
    let volumes: Vec<VolumeInfo> = disks
        .iter()
        .filter_map(|disk| {
            let mount_str = disk.mount_point().to_string_lossy();

            // Filter out internal macOS system helper mounts that have the 'nobrowse' flag:
            // - /System/Volumes/Data (APFS firmlinked storage, already transparently reachable via /)
            // - /System/Volumes/VM, Preboot, Update, xarts, Hardware, iSCPreboot
            // - /Volumes/Recovery
            // - Simulator runtimes and virtual filesystems
            if mount_str.starts_with("/System/Volumes")
                || mount_str.starts_with("/dev")
                || mount_str.starts_with("/Library/Developer/CoreSimulator")
                || mount_str == "/Volumes/Recovery"
                || mount_str.contains("auto_home")
            {
                return None;
            }

            let total = disk.total_space();
            let available = disk.available_space();
            let mut name = disk.name().to_string_lossy().to_string();

            if name.is_empty() && mount_str == "/" {
                name = "Macintosh HD".to_string();
            }

            Some(VolumeInfo {
                name,
                mount_point: mount_str.to_string(),
                fs_type: disk.file_system().to_string_lossy().to_string(),
                total_bytes: total,
                available_bytes: available,
                used_bytes: total.saturating_sub(available),
                is_removable: disk.is_removable(),
                is_read_only: disk.is_read_only(),
            })
        })
        .collect();
    Ok(volumes)
}
