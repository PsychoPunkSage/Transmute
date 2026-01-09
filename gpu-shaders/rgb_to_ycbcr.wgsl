// RGB to YCbCr color space conversion (JPEG color space)
// Runs on GPU for parallel processing of all pixels

struct ImageParams {
    width: u32,
    height: u32,
}

@group(0) @binding(0) var<uniform> params: ImageParams;
@group(0) @binding(1) var<storage, read> input_rgb: array<u32>;  // Packed RGB888 = 0xRRGGBB = [8 bits Red] + [8 bits Green] + [8 bits Blue] = 32 bit = 1 pixel color
@group(0) @binding(2) var<storage, read_write> output_ycbcr: array<vec4<f32>>;

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
    
    // Unpack RGB from u32 (assumes RGB888 format: 0xRRGGBB)
    let packed = input_rgb[pixel_index];
    let r = f32((packed >> 16u) & 0xFFu);
    let g = f32((packed >> 8u) & 0xFFu);
    let b = f32(packed & 0xFFu);
    
    let rgb = vec3<f32>(r, g, b);
    let ycbcr = rgb_to_ycbcr(rgb);
    
    // Write YCbCr values
    output_ycbcr[pixel_index] = vec4<f32>(ycbcr, 1.0);
}
