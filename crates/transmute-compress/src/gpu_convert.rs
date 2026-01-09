use bytemuck::{Pod, Zeroable};
use transmute_common::{Error, Result};
use wgpu::{Device, Queue};

/// GPU context for color space conversion
pub struct GpuColorConverter {
    device: Device,
    queue: Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
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
        })
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

        // Create GPU buffers
        let params = ImageParams { width, height };
        let params_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Params Buffer"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let input_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Input RGB Buffer"),
                contents: bytemuck::cast_slice(&packed_rgb),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let output_size = (pixel_count * 4 * std::mem::size_of::<f32>()) as u64; // vec4<f32>
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output YCbCr Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create staging buffer for readback
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Color Convert Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buffer.as_entire_binding(),
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

        // Copy to staging buffer
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, output_size);

        self.queue.submit(Some(encoder.finish()));

        // Read back results
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);
        pollster::block_on(rx)
            .map_err(|_| Error::GpuError("Failed to receive map result".into()))?
            .map_err(|e| Error::GpuError(format!("Buffer mapping failed: {:?}", e)))?;

        let data = buffer_slice.get_mapped_range();
        let ycbcr_vec4: Vec<[f32; 4]> = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        staging_buffer.unmap();

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
    use crate::transmute_core::GpuContext;

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
