// src/omnixelerator/resource_monitor.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXELERATOR]Xyn>=====S===t===u===d===i===o===s======[R|$>

use crate::omnixtracker::omnixerror::OmniXError;
use crate::constants::*;
use opencl3::{
    device::{CL_DEVICE_TYPE_GPU, Device as OpenCLDevice},
    platform::get_platforms,
};
use sysinfo::{System, SystemExt, CpuExt, ComponentExt};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use nvml_wrapper::{Device, Nvml};
use parking_lot::{Mutex, RwLock};
use log::{info, warn, error};
use std::sync::Arc;
use tokio::task;

/// Represents system hardware details.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SystemHardware {
    pub total_memory: usize,
    pub gpus: Vec<GPUInfo>,
    pub cpu_cores: usize,
}

impl SystemHardware {
    /// Initializes SystemHardware by querying the system.
    pub async fn new(nvml: &Arc<Nvml>) -> Result<Self, OmniXError> { // Changed to OmniXError
        let total_memory = Self::get_total_memory()?;
        let gpus = Self::get_gpus(nvml)?;
        let cpu_cores = num_cpus::get();
        Ok(Self {
            total_memory,
            gpus,
            cpu_cores,
        })
    }

    /// Retrieves the total system memory.
    fn get_total_memory() -> Result<usize, OmniXError> { // Changed to OmniXError
        let mut system = System::new_all();
        system.refresh_memory();
        Ok(system.total_memory() as usize * 1024) // Convert from KB to bytes
    }

    /// Retrieves GPU information using NVML.
    fn get_gpus(nvml: &Arc<Nvml>) -> Result<Vec<GPUInfo>, OmniXError> { // Changed to OmniXError
        let device_count = nvml.device_count().map_err(|e| OmniXError::NvmlError(e.to_string()))?; // Changed to OmniXError
        let mut gpus = Vec::new();
        for i in 0..device_count {
            let device = nvml.device_by_index(i).map_err(|e| OmniXError::NvmlError(e.to_string()))?; // Changed to OmniXError
            let opencl_device = Self::get_opencl_device(i)?;
            gpus.push(GPUInfo { device, opencl_device });
        }
        Ok(gpus)
    }

    /// Retrieves the corresponding OpenCL device for a given GPU index.
    fn get_opencl_device(gpu_index: usize) -> Result<Option<OpenCLDevice>, OmniXError> { // Changed to OmniXError
        let platforms = get_platforms().map_err(|e| OmniXError::OpenCLError(e.to_string()))?; // Changed to OmniXError
        for platform in platforms {
            if let Ok(devices) = platform.get_devices(CL_DEVICE_TYPE_GPU) {
                if let Some(device) = devices.get(gpu_index) {
                    return Ok(Some(device.clone()));
                }
            }
        }
        Ok(None)
    }

    pub fn print_summary(&self) {
        println!("System Hardware Summary:");
        println!("-------------------------");
        println!("Total Memory: {} GB", self.total_memory / (1024 * 1024 * 1024));
        println!("CPU Cores: {}", self.cpu_cores);
        println!("GPUs: {}", self.gpus.len());
        for (i, gpu) in self.gpus.iter().enumerate() {
            println!("  GPU {}:", i);
            if let Ok(name) = gpu.device.name() {
                println!("    Name: {}", name);
            }
            if let Ok(memory) = gpu.device.memory_info() {
                println!("    Memory: {} GB", memory.total / (1024 * 1024 * 1024));
            }
            if let Ok(cc) = gpu.device.cuda_compute_capability() {
                println!("    Compute Capability: {}.{}", cc.0, cc.1);
            }
            println!("    OpenCL Support: {}", if gpu.opencl_device.is_some() { "Yes" } else { "No" });
        }
    }
}

/// Represents information about a GPU.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GPUInfo {
    pub device: Device,
    pub opencl_device: Option<OpenCLDevice>,
}

/// Monitors system resources.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ResourceMonitor {
    pub gpu_temperature: Vec<f32>,
    pub available_memory: usize,
    pub cpu_temperature: f32,
    pub cpu_usage: Vec<f32>,
    pub gpu_usage: Vec<f32>,
}

impl ResourceMonitor {
    /// Initializes a new ResourceMonitor.
    pub fn new(hardware: &SystemHardware) -> Self {
        Self {
            gpu_temperature: vec![0.0; hardware.gpus.len()],
            available_memory: 0,
            cpu_temperature: 0.0,
            cpu_usage: vec![0.0; hardware.cpu_cores],
            gpu_usage: vec![0.0; hardware.gpus.len()],
        }
    }

    /// Updates the resource metrics.
    pub async fn update(&mut self, nvml: &Arc<Nvml>, hardware: &SystemHardware) -> Result<(), OmniXError> { // Changed to OmniXError
        let (memory_info, cpu_info, gpu_info) = tokio::try_join!(
            task::spawn_blocking(Self::get_memory_info),
            task::spawn_blocking(|| Self::get_cpu_info(hardware.cpu_cores)),
            Self::get_gpu_info(nvml, &hardware.gpus)
        )?;
        self.available_memory = memory_info;
        self.cpu_usage = cpu_info.0;
        self.cpu_temperature = cpu_info.1;
        self.gpu_usage = gpu_info.0;
        self.gpu_temperature = gpu_info.1;
        Ok(())
    }

    fn get_memory_info() -> Result<usize, OmniXError> { // Changed to OmniXError
        let mut system = System::new_all();
        system.refresh_memory();
        Ok(system.available_memory() as usize * 1024) // Convert from KB to bytes
    }

    fn get_cpu_info(cpu_cores: usize) -> Result<(Vec<f32>, f32), OmniXError> { // Changed to OmniXError
        let mut system = System::new_all();
        system.refresh_cpu();
        system.refresh_components_list();
        
        let cpu_usage: Vec<f32> = system.cpus().iter().take(cpu_cores)
            .map(|cpu| cpu.cpu_usage() as f32)
            .collect();
        let cpu_temp = system.components()
            .iter()
            .find(|component| component.label().contains("CPU"))
            .map(|cpu| cpu.temperature() as f32)
            .unwrap_or(0.0);
        Ok((cpu_usage, cpu_temp))
    }

    async fn get_gpu_info(nvml: &Arc<Nvml>, gpus: &[GPUInfo]) -> Result<(Vec<f32>, Vec<f32>), OmniXError> { // Changed to OmniXError
        let mut gpu_usage = Vec::with_capacity(gpus.len());
        let mut gpu_temp = Vec::with_capacity(gpus.len());
        for gpu in gpus {
            let utilization = gpu.device.utilization_rates()
                .map_err(|e| OmniXError::NvmlError(e.to_string()))?; // Changed to OmniXError
            gpu_usage.push(utilization.gpu as f32);
            let temperature = gpu.device.temperature()
                .map_err(|e| OmniXError::NvmlError(e.to_string()))?; // Changed to OmniXError
            gpu_temp.push(temperature as f32);
        }
        Ok((gpu_usage, gpu_temp))
    }

    pub fn print_summary(&self) {
        println!("Resource Monitor Summary:");
        println!("-------------------------");
        println!("Available Memory: {} GB", self.available_memory / (1024 * 1024 * 1024));
        println!("CPU Usage:");
        for (i, usage) in self.cpu_usage.iter().enumerate() {
            println!("  Core {}: {:.2}%", i, usage);
        }
        println!("CPU Temperature: {:.2}°C", self.cpu_temperature);
        println!("GPU Usage:");
        for (i, (usage, temp)) in self.gpu_usage.iter().zip(self.gpu_temperature.iter()).enumerate() {
            println!("  GPU {}: {:.2}% (Temperature: {:.2}°C)", i, usage, temp);
        }
    }
}