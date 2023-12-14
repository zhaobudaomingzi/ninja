use serde::Serialize;
use std::sync::OnceLock;
use sysinfo::{LoadAvg, NetworkExt, System, SystemExt};

static mut SYSINFO: OnceLock<System> = OnceLock::new();

fn get_system() -> &'static mut System {
    unsafe {
        SYSINFO.get_or_init(|| {
            let system = System::new_all();
            sysinfo::set_open_files_limit(0);
            system
        });
        SYSINFO.get_mut().expect("get system failed")
    }
}

pub(super) fn get_info() -> SysInfo {
    let system = get_system();

    // Disk info
    system.refresh_disks();
    let mut disks = Vec::new();
    for disk in system.disks() {
        disks.push(serde_json::to_value(disk).unwrap())
    }

    // Cpu info
    system.refresh_cpu();
    let mut cpus = Vec::new();
    for cpu in system.cpus() {
        cpus.push(serde_json::to_value(cpu).unwrap())
    }

    // Memory info
    system.refresh_memory();
    let memory_info = MemoryInfo {
        total_memory: system.total_memory(),
        free_memory: system.free_memory(),
        used_memory: system.used_memory(),
        available_memory: system.available_memory(),
    };

    // Swap info
    let swap_info = SwapInfo {
        total_swap: system.total_swap(),
        free_swap: system.free_swap(),
        used_swap: system.used_swap(),
    };

    // Network info
    system.refresh_networks();
    let mut network_info = Vec::new();
    for (interface_name, data) in system.networks() {
        network_info.push(NetworkInfo {
            interface_name: interface_name.to_string(),
            received: data.received(),
            transmitted: data.transmitted(),
        });
    }

    SysInfo {
        name: system.long_os_version(),
        os_version: system.os_version(),
        kernel_version: system.kernel_version(),
        uptime: system.uptime(),
        load_average: system.load_average(),
        cpu: cpus,
        memory: memory_info,
        swap: swap_info,
        disk: disks,
        network: network_info,
    }
}

#[derive(Serialize)]
pub(super) struct SysInfo {
    name: Option<String>,
    os_version: Option<String>,
    kernel_version: Option<String>,
    uptime: u64,
    load_average: LoadAvg,
    cpu: Vec<serde_json::Value>,
    memory: MemoryInfo,
    swap: SwapInfo,
    disk: Vec<serde_json::Value>,
    network: Vec<NetworkInfo>,
}

#[derive(Serialize)]
pub(super) struct MemoryInfo {
    total_memory: u64,
    free_memory: u64,
    used_memory: u64,
    available_memory: u64,
}

#[derive(Serialize)]
pub(super) struct SwapInfo {
    total_swap: u64,
    free_swap: u64,
    used_swap: u64,
}

#[derive(Serialize)]
pub(super) struct NetworkInfo {
    interface_name: String,
    received: u64,
    transmitted: u64,
}
