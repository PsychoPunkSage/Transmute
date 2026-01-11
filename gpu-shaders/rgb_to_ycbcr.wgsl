// RGB to YCbCr color space conversion (JPEG color space)
// Runs on GPU for parallel processing of all pixels

struct ImageParams {
    width: u32,
    height: u32,
}

@group(0) @binding(0) var<uniform> params: ImageParams;
@group(0) @binding(1) var<storage, read> input_rgb: array<vec4<u32>>;  // RGBA as 4 u8 packed efficiently
@group(0) @binding(2) var<storage, read_write> output_ycbcr: array<vec4<u32>>;  // Output as u32 packed - skip f32 entirely

// ITU-R BT.601 conversion matrix (JPEG standard)
fn rgb_to_ycbcr(rgb: vec3<f32>) -> vec3<f32> {
    // Human Care more about Brightness (Y) and less about color (Cb/Cr) || So, keep Y - sharp, Blur Cb/Cr, Shrink file sizes massively
    let y  =  0.299    * rgb.r + 0.587    * rgb.g + 0.114    * rgb.b;
    let cb = -0.168736 * rgb.r - 0.331264 * rgb.g + 0.5      * rgb.b + 128.0;
    let cr =  0.5      * rgb.r - 0.418688 * rgb.g - 0.081312 * rgb.b + 128.0;
    
    return vec3<f32>(y, cb, cr);
}

@compute @workgroup_size(16, 16, 1) // workgroup_size = 256
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    // Bounds check
    if (x >= params.width || y >= params.height) {
        return;
    }

    let pixel_index = y * params.width + x;

    // Read RGBA directly as vec4<u32> (each component 0-255)
    let rgba = input_rgb[pixel_index];

    // Convert to f32 for processing
    let rgb = vec3<f32>(f32(rgba.r), f32(rgba.g), f32(rgba.b));
    let ycbcr_f = rgb_to_ycbcr(rgb);

    // Clamp and convert back to u32 - output as u8 values in u32
    let y_u = u32(clamp(ycbcr_f.x, 0.0, 255.0));
    let cb_u = u32(clamp(ycbcr_f.y, 0.0, 255.0));
    let cr_u = u32(clamp(ycbcr_f.z, 0.0, 255.0));

    // Pack as vec4<u32> - matches u8 layout expected by mozjpeg
    output_ycbcr[pixel_index] = vec4<u32>(y_u, cb_u, cr_u, 0u);
}
