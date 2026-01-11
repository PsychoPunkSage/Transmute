use crate::{Error, Result};
use wgpu::{AdapterInfo, Device, Instance, PowerPreference, Queue, RequestAdapterOptions};

/// GPU context manager with automatic fallback
pub struct GpuContext {
    pub device: Device,
    pub queue: Queue,
    pub adapter_info: AdapterInfo,
}

impl GpuContext {
    /// Initialize GPU context with best available adapter
    pub fn new() -> Result<Self> {
        pollster::block_on(Self::new_async())
    }

    async fn new_async() -> Result<Self> {
        tracing::info!("Initializing GPU context...");

        // 1. List out all the GPU device
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 2. Get the STRONGEST Adapter/GPU
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| Error::GpuError("Failed to find suitable GPU adapter".into()))?;

        let adapter_info = adapter.get_info();
        tracing::info!(
            "Selected GPU: {} ({:?})",
            adapter_info.name,
            adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Transmute GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::GpuError(format!("Failed to create device: {}", e)))?;

        Ok(Self {
            device,
            queue,
            adapter_info,
        })
    }

    /// Check if GPU has sufficient compute capabilities
    pub fn supports_compute(&self) -> bool {
        self.device.features().contains(wgpu::Features::empty())
    }

    /// Get available GPU memory (estimate)
    pub fn estimated_memory_mb(&self) -> Option<u64> {
        // Not directly exposed by wgpu, return None for now
        // Will be implemented in later phases with vendor-specific queries
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_initialization() {
        let ctx = GpuContext::new();
        assert!(ctx.is_ok(), "GPU context should initialize");

        if let Ok(ctx) = ctx {
            println!("GPU: {}", ctx.adapter_info.name);
        }
    }
}
