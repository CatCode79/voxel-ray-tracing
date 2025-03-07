@group(0) @binding(0) var result_texture_: texture_2d<f32>;
@group(0) @binding(1) var denoised_texture_: texture_storage_2d<rgba8unorm, write>;

const KERNEL_SIZE: i32 = 1;

fn guassian_color_weight(x: vec3f, y: vec3f) -> f32 {
    let dist_sq = f32(dot(x - y, x - y));
    // (2.0 * sigma * sigma), but sigma is one, so 2.0
    return exp(-dist_sq / 2.0f);
}

fn guassian_spatial_weight(x: vec2i, y: vec2i) -> f32 {
    let dist_sq = f32(dot(x - y, x - y));
    // 2.0 * sigma * sigma, but sigma space is one, so 2.0
    return exp(-dist_sq / 2.0f);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) inv_id: vec3u) {
    let screen_pos = vec2i(inv_id.xy);
    let center_color: vec3f = textureLoad(result_texture_, screen_pos, 0).rgb;
    var result = vec3(0.0);
    var weight_sum = 0.0;
    
    var i: i32 = -KERNEL_SIZE;
    while i <= KERNEL_SIZE {
        var j: i32 = -KERNEL_SIZE;
        while j <= KERNEL_SIZE {
            let current_coords: vec2i = screen_pos + vec2(i, j);
            let current_color: vec3f = textureLoad(result_texture_, current_coords, 0).rgb;
            
            let color_weight: f32 = guassian_color_weight(center_color, current_color);
            let spatial_weight: f32 = guassian_spatial_weight(screen_pos, current_coords);
            let weight: f32 = color_weight * spatial_weight;
            
            result += current_color * weight;
            weight_sum += weight;
            
            j += 1;
        }
        i += 1;
    }

    textureStore(denoised_texture_, screen_pos, vec4(result / weight_sum, 1.0));
}
