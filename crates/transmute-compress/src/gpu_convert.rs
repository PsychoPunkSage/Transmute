use bytemuck::{Pod, Zeroable};
use std::cell::RefCell;
use transmute_common::{Error, Result};
use wgpu::{Device, Queue};

/// Reusable GPU buffer pool for a specific size
struct BufferPool {
    input_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    capacity: usize, // pixel count
}

/// GPU context for color space conversion with buffer pooling
pub struct GpuColorConverter {
    device: Device,
    queue: Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    buffer_pool: RefCell<Option<BufferPool>>,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ImageParams {
    width: u32,
    height: u32,
}

impl GpuColorConverter {
    /// Initialize GPU color converter with shader
    pub fn new(device: Device, queue: Queue) -> Result<Self> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("RGB to YCbCr Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../gpu-shaders/rgb_to_ycbcr.wgsl").into(),
            ),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Color Convert Bind Group Layout"),
            entries: &[
                // Uniform buffer: image dimensions
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Storage buffer: input RGB data
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Storage buffer: output YCbCr data
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Convert Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Color Convert Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            buffer_pool: RefCell::new(None),
        })
    }

    /// Get or create buffer pool for given pixel count
    /// Reuses buffers if size matches, creates new ones if needed
    fn get_or_create_buffers(&self, pixel_count: usize) -> BufferPool {
        let mut pool = self.buffer_pool.borrow_mut();

        // Reuse if capacity matches
        if let Some(existing) = pool.as_ref() {
            if existing.capacity == pixel_count {
                tracing::debug!("Reusing buffer pool for {} pixels", pixel_count);
                return BufferPool {
                    input_buffer: existing.input_buffer.clone(),
                    output_buffer: existing.output_buffer.clone(),
                    staging_buffer: existing.staging_buffer.clone(),
                    params_buffer: existing.params_buffer.clone(),
                    capacity: existing.capacity,
                };
            }
        }

        // Create new buffers
        tracing::debug!("Creating new buffer pool for {} pixels", pixel_count);
        let input_size = (pixel_count * std::mem::size_of::<u32>()) as u64;
        let output_size = (pixel_count * 4 * std::mem::size_of::<f32>()) as u64;

        let input_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pooled Input RGB Buffer"),
            size: input_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pooled Output YCbCr Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pooled Staging Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let params_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pooled Params Buffer"),
            size: std::mem::size_of::<ImageParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let new_pool = BufferPool {
            input_buffer,
            output_buffer,
            staging_buffer,
            params_buffer,
            capacity: pixel_count,
        };

        *pool = Some(BufferPool {
            input_buffer: new_pool.input_buffer.clone(),
            output_buffer: new_pool.output_buffer.clone(),
            staging_buffer: new_pool.staging_buffer.clone(),
            params_buffer: new_pool.params_buffer.clone(),
            capacity: new_pool.capacity,
        });

        new_pool
    }

    /// Convert RGB image data to YCbCr on GPU
    /// Returns YCbCr data as flat Vec<f32> [Y, Cb, Cr, Y, Cb, Cr, ...]
    pub fn rgb_to_ycbcr(&self, rgb_data: &[u8], width: u32, height: u32) -> Result<Vec<f32>> {
        let pixel_count = (width * height) as usize;

        tracing::debug!("GPU: Converting {}x{} RGB→YCbCr", width, height);

        // Pack RGB888 into u32 array (0xRRGGBB)
        let mut packed_rgb = Vec::with_capacity(pixel_count);
        for chunk in rgb_data.chunks_exact(3) {
            let r = chunk[0] as u32;
            let g = chunk[1] as u32;
            let b = chunk[2] as u32;
            packed_rgb.push((r << 16) | (g << 8) | b);
        }

        // Get or create pooled buffers
        let buffers = self.get_or_create_buffers(pixel_count);

        // Update buffer contents
        let params = ImageParams { width, height };
        self.queue.write_buffer(&buffers.params_buffer, 0, bytemuck::bytes_of(&params));
        self.queue.write_buffer(&buffers.input_buffer, 0, bytemuck::cast_slice(&packed_rgb));

        let output_size = (pixel_count * 4 * std::mem::size_of::<f32>()) as u64;

        // Create bind group (using pooled buffers)
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Convert Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.output_buffer.as_entire_binding(),
                },
            ],
        });

        // Dispatch compute shader
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Color Convert Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Color Convert Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            // Calculate workgroups (16x16 workgroup size from shader)
            let workgroups_x = (width + 15) / 16;
            let workgroups_y = (height + 15) / 16;
            compute_pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        // Copy to staging buffer (using pooled buffers)
        encoder.copy_buffer_to_buffer(&buffers.output_buffer, 0, &buffers.staging_buffer, 0, output_size);

        self.queue.submit(Some(encoder.finish()));

        // Read back results - map buffer and wait for completion
        let buffer_slice = buffers.staging_buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        // Poll and wait for completion
        let _ = self.device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(std::time::Duration::from_secs(5))
        });
        pollster::block_on(rx)
            .map_err(|_| Error::GpuError("Failed to receive buffer mapping result".into()))?
            .map_err(|e| Error::GpuError(format!("Buffer mapping failed: {:?}", e)))?;

        let data = buffer_slice.get_mapped_range();
        let ycbcr_vec4: Vec<[f32; 4]> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        buffers.staging_buffer.unmap();

        // Flatten vec4 to vec3 (ignore alpha channel)
        let mut ycbcr_flat = Vec::with_capacity(pixel_count * 3);
        for pixel in ycbcr_vec4 {
            ycbcr_flat.push(pixel[0]); // Y
            ycbcr_flat.push(pixel[1]); // Cb
            ycbcr_flat.push(pixel[2]); // Cr
        }

        tracing::debug!("GPU: Conversion complete");
        Ok(ycbcr_flat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use transmute_common::gpu::GpuContext;

    #[test]
    fn test_gpu_color_conversion() {
        let gpu_ctx = GpuContext::new().unwrap();
        let converter = GpuColorConverter::new(gpu_ctx.device, gpu_ctx.queue).unwrap();

        // Create test RGB data (4 pixels: red, green, blue, white)
        let rgb_data: Vec<u8> = vec![
            255, 0, 0, // Red
            0, 255, 0, // Green
            0, 0, 255, // Blue
            255, 255, 255, // White
        ];

        let ycbcr = converter.rgb_to_ycbcr(&rgb_data, 2, 2).unwrap();

        // Verify YCbCr values (approximately)
        // Red: Y≈76, Cb≈84, Cr≈255
        assert!((ycbcr[0] - 76.0).abs() < 2.0); // Y
        assert!((ycbcr[1] - 84.0).abs() < 2.0); // Cb
        assert!((ycbcr[2] - 255.0).abs() < 2.0); // Cr
    }
}
