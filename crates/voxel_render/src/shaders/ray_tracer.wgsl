struct CamData {
    pos: vec3f,
    inv_view_mat: mat4x4f,
    inv_proj_mat: mat4x4f,
    proj_size: vec2f,
}

struct FrameData {
    counter: u32,
    cumulator: u32,
}

struct Settings {
    max_ray_bounces: u32,
    samples_per_pixel: u32,
    sun_intensity: f32,
    sky_color: vec3f,
    sun_pos: vec3f,
}

struct World {
    min: vec3f,
    size: f32,
}

fn get_bits(field: u32, len: u32, offset: u32) -> u32 {
    let mask = ~(~0u << len) << offset;
    return (field & mask) >> offset;
}

fn node_voxel(node_idx: u32) -> u32 {
    return get_bits(nodes_[node_idx], 8u, 0u);
}
fn node_is_split(node_idx: u32) -> bool {
    return get_bits(nodes_[node_idx], 1u, 31u) == 1u;
}
fn node_child(node_idx: u32, child: u32) -> u32 {
    return get_bits(nodes_[node_idx], 30u, 0u) * 8u + 1u + child;
}

struct Material {
    color: vec3f,
    empty: u32,
    scatter: f32,
    emission: f32,
    polish_bounce_chance: f32,
    polish_color: vec3f,
    polish_scatter: f32,
}

@group(0) @binding(0) var normal_texture_: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> cam_data_: CamData;
@group(0) @binding(2) var<storage, read> nodes_: array<u32>;
@group(0) @binding(3) var<uniform> world_: World;

struct Ray {
    origin: vec3f,
    dir: vec3f,
}

struct FoundNode {
    idx: u32,
    min: vec3f,
    max: vec3f,
    center: vec3f,
    size: f32,
}

fn find_node(pos: vec3f) -> FoundNode {
    var size = f32(world_.size);
    var center = world_.min + vec3f(size * 0.5);
    var node_idx = 0u;

    loop {
        if (!node_is_split(node_idx)) {
            var out: FoundNode;
            out.idx = node_idx;
            out.min = vec3f(center) - vec3f(size * 0.5);
            out.max = vec3f(center) + vec3f(size * 0.5);
            out.center = vec3f(center);
            out.size = size;
            return out;
        }
        size *= 0.5;

        let gt: vec3<bool> = pos >= center;
        let child_idx =
            u32(gt.x) << 0u |
            u32(gt.y) << 1u |
            u32(gt.z) << 2u;

        node_idx = node_child(node_idx, child_idx);
        let child_dir = vec3f(gt) * 2.0 - vec3f(1.0);
        center += (size * 0.5) * child_dir;
    }
    // this shouldn't happen, even if `pos` is outside of the world bounds
    var out: FoundNode;
    return out;
}

fn ray_normal(start_ray: Ray) -> vec3f {
    let dir = start_ray.dir;
    let mask = vec3f(f32(dir.x >= 0.0), f32(dir.y >= 0.0), f32(dir.z >= 0.0));
    let imask = 1.0 - mask;

    var ray_pos = start_ray.origin;

    let world_min = world_.min;
    let world_max = world_min + vec3f(world_.size);

    if (any(ray_pos <= world_min) | any(ray_pos >= world_max)) {
        return vec3f(0.0);
    }

    // length of a line in same direction as the ray,
    // that travels 1 unit in the X, Y, Z

    // dir - normilized --- x^2 + y^2 + z^2 = 1
    let unit_step_size = vec3f(
        sqrt(1.0 + (dir.y / dir.x) * (dir.y / dir.x) + (dir.z / dir.x) * (dir.z / dir.x)),
        sqrt(1.0 + (dir.x / dir.y) * (dir.x / dir.y) + (dir.z / dir.y) * (dir.z / dir.y)),
        sqrt(1.0 + (dir.x / dir.z) * (dir.x / dir.z) + (dir.y / dir.z) * (dir.y / dir.z)),
    );

    var voxel: u32;
    var norm: vec3f;

    var iter_count: u32 = 0u;
    while iter_count < 128u {  // Probably is a limiter to avoid infinite loop TODO: farla parametrica, è la distanza dalla camera
        iter_count += 1u;

        let found_node = find_node(ray_pos); // the most child one
        voxel = node_voxel(found_node.idx); // just voxel - most time air

        if (voxel != 0u) { // not air, so return it
            break;
        }
        let node_min = vec3f(found_node.min);
        let node_max = vec3f(found_node.max);

        let axis_dist = (
            (ray_pos - node_min) * imask + (node_max - ray_pos) * mask
        ) * unit_step_size;

        var step: f32;

        if (axis_dist.x == 0.0) {
            if (axis_dist.y == 0.0) {
                step = axis_dist.z;
            } else if (axis_dist.z == 0.0) {
                step = axis_dist.y;
            } else {
                step = min(axis_dist.y, axis_dist.z);
            }
        } else {
            if (axis_dist.y == 0.0) {
                if (axis_dist.z == 0.0) {
                    step = axis_dist.x;
                } else {
                    step = min(axis_dist.x, axis_dist.z);
                }
            } else {
                if (axis_dist.z == 0.0) {
                    step = min(axis_dist.y, axis_dist.x);
                } else {
                    step = min(axis_dist.x, min(axis_dist.y, axis_dist.z));
                }
            }
        }

        norm = vec3f(f32(step == axis_dist.x), f32(step == axis_dist.y), f32(step == axis_dist.z));

        if (any(ray_pos < world_min) | any(ray_pos >= world_max)) {
            return norm;
        } // out of bounds
    } // return not air OR max steps already !!!!!!!!!!!

    return norm;
}

fn create_ray_from_screen(screen_pos: vec2i) -> Ray {
    let x = (f32(screen_pos.x) * 2.0) / cam_data_.proj_size.x - 1.0;
    let y = (f32(screen_pos.y) * 2.0) / cam_data_.proj_size.y - 1.0;
    let clip_coords = vec4(x, -y, -1.0, 1.0);
    let eye_coords0 = clip_coords * cam_data_.inv_proj_mat;
    let eye_coords = vec4(eye_coords0.xy, -1.0, 0.0);
    let ray_world = normalize((eye_coords * cam_data_.inv_view_mat).xyz);

    var ray: Ray;
    ray.origin = cam_data_.pos;
    ray.dir = ray_world;
    return ray;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) inv_id: vec3u) {
    let screen_pos = vec2i(inv_id.xy);
    let ray = create_ray_from_screen(screen_pos);
    let normal = ray_normal(ray);

    textureStore(normal_texture_, screen_pos, vec4(normal, 0.0));
}
