//= IMPORTS ==================================================================

use crate::open_simplex::{init_gradients, NoiseMap};

use voxel_math::aabb::Aabb;
use voxel_render::{Node, Voxel};

use glam::{ivec3, vec2, vec3, IVec3, Vec3};

use std::ops::Range;

//= WORLD ====================================================================

#[derive(Clone, Debug)]
pub enum WorldErr {
    OutOfBounds,
}

struct FoundNode {
    idx: u32,
    depth: u32,
    center: IVec3,
    size: u32,
}

/// The structure that holds the entire interactable world, representing all voxels via a SVO.
#[derive(Clone, Default)]
pub struct World {
    pub min: IVec3,
    pub size: u32,
    pub max_nodes: u32,
    pub max_depth: u32,
    start_search: u32,
    last_used_node: u32,

    // Note: Removing items from the Vec is not good since
    // some nodes may point to other nodes by index.
    nodes: Vec<Node>,
}

/// Create and clear worlds
impl World {
    pub fn new(max_buffer_sizes: u32) -> Self {
        let max_nodes = max_buffer_sizes / size_of::<Node>() as u32;
        let world_depth = 9;
        let world_size = 1 << world_depth;

        init_gradients();

        let mut nodes = vec![Node::ZERO; max_nodes as usize];
        nodes[0] = Node::new_leaf(Voxel::AIR);
        Self {
            min: IVec3::ZERO,
            size: world_size,
            max_nodes,
            max_depth: world_depth,
            start_search: 1,
            last_used_node: 0,
            nodes,
        }
    }

    #[inline(always)]
    pub fn min(&self) -> IVec3 {
        self.min
    }
    #[inline(always)]
    pub fn max(&self) -> IVec3 {
        self.min + IVec3::splat(self.size as i32)
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes[0..=self.last_used_node as usize]
    }

    pub fn set_max_depth(&mut self, max_depth: u32) {
        self.max_depth = max_depth;
        self.size = 1 << max_depth;
    }

    pub fn last_used_node(&self) -> u32 {
        self.last_used_node
    }

    pub fn clear_node(&mut self, idx: u32) {
        self.free_node(idx);
        self.nodes[idx as usize] = Node::new_leaf(Voxel::AIR);
    }

    pub fn clear(&mut self) {
        for node in &mut self.nodes {
            node.set_used_flag(false);
        }
        self.nodes[0] = Node::new_leaf(Voxel::AIR);
        self.start_search = 1;
        self.last_used_node = 0;
    }
}
/// Find and mutate the SVO nodes that make up the world.
impl World {
    pub fn check_bounds(&self, pos: IVec3) -> Result<(), WorldErr> {
        let in_bounds = (pos.cmpge(self.min())).all() && (pos.cmplt(self.max())).all();
        in_bounds.then(|| ()).ok_or(WorldErr::OutOfBounds)
    }

    fn find_node(&self, pos: IVec3, max_depth: u32) -> Result<FoundNode, WorldErr> {
        self.check_bounds(pos)?;

        let mut center = self.min + IVec3::splat(self.size as i32 / 2);
        let mut size = self.size;
        let mut node_idx = 0;
        let mut depth: u32 = 0;

        loop {
            let node = self.get_node(node_idx);
            if !node.is_split() || depth == max_depth {
                return Ok(FoundNode {
                    idx: node_idx,
                    depth,
                    center,
                    size,
                });
            }
            size /= 2;

            let gt = ivec3(
                (pos.x >= center.x) as i32,
                (pos.y >= center.y) as i32,
                (pos.z >= center.z) as i32,
            );
            let child_idx = (gt.x as u32) << 0 | (gt.y as u32) << 1 | (gt.z as u32) << 2;
            node_idx = node.get_child(child_idx);
            let child_dir = gt * 2 - IVec3::ONE;
            center += IVec3::splat(size as i32 / 2) * child_dir;
            depth += 1;
        }
    }

    #[inline(always)]
    pub fn get_node(&self, idx: u32) -> Node {
        self.nodes[idx as usize]
    }

    #[inline(always)]
    pub fn swap_nodes(&mut self, a: u32, b: u32) {
        let b_node = self.nodes[b as usize];
        self.nodes[b as usize] = self.nodes[a as usize];
        self.nodes[a as usize] = b_node;
    }

    #[inline(always)]
    pub fn mut_node(&mut self, idx: u32) -> &mut Node {
        &mut self.nodes[idx as usize]
    }

    #[inline(always)]
    pub fn free_nodes(&mut self, start: u32) {
        if start < self.start_search {
            self.start_search = start;
        }
        for idx in start..start + 8 {
            self.free_node(idx);
        }
    }

    #[inline(always)]
    pub fn free_node(&mut self, idx: u32) {
        self.nodes[idx as usize].set_used_flag(false);
        if self.nodes[idx as usize].is_split() {
            self.free_nodes(self.nodes[idx as usize].first_child());
        }
    }

    fn new_nodes(&mut self, voxel: Voxel) -> u32 {
        static NEW_NODES: [Node; 1024] = [Node::ZERO; 1024];

        let mut result = self.start_search;
        if result + 7 >= self.nodes.len() as u32 {
            // result can at most be nodes.len(),
            // so adding 8 more nodes should make result valid,
            // but I add 1024 nodes here because it's very likely
            // to want to allocate more nodes very soon after
            self.nodes.extend(&NEW_NODES);
        }

        while self.get_node(result).is_used() {
            result += 8;
            if result + 7 >= self.nodes.len() as u32 {
                self.nodes.extend(&NEW_NODES);
            }
        }
        self.start_search = result + 8;

        for idx in result..result + 8 {
            self.nodes[idx as usize] = Node::new_leaf(voxel);
        }
        if result > self.last_used_node.saturating_sub(7) {
            self.last_used_node = result + 7;
        }
        result
    }
}

#[derive(Clone, Copy)]
pub struct NodeSeq {
    pub idx: u32,
    pub count: u8,
}

/// High-level voxel-based manipulation.
impl World {
    pub fn get_voxel(&self, pos: IVec3) -> Result<Voxel, WorldErr> {
        let FoundNode { idx, .. } = self.find_node(pos, self.max_depth)?;
        Ok(self.get_node(idx).get_voxel())
    }

    pub fn set_voxel(&mut self, pos: IVec3, voxel: Voxel) -> Result<Vec<NodeSeq>, WorldErr> {
        let target_depth = self.max_depth;
        let FoundNode {
            mut idx,
            depth,
            mut center,
            mut size,
            ..
        } = self.find_node(pos, target_depth)?;
        let old_voxel = self.get_node(idx).get_voxel();

        let mut result: Vec<NodeSeq> = vec![];
        result.push(NodeSeq { idx, count: 1 });

        // If depth is less than target_depth,
        // the SVO doesn't go to desired depth, so we must split until it does
        for _ in depth..target_depth {
            let first_child = self.new_nodes(old_voxel);

            self.mut_node(idx).set_split_flag(true);
            self.mut_node(idx).set_first_child(first_child);
            result.push(NodeSeq {
                idx: first_child,
                count: 8,
            });

            size /= 2;

            let gt = ivec3(
                (pos.x >= center.x) as i32,
                (pos.y >= center.y) as i32,
                (pos.z >= center.z) as i32,
            );
            let child_idx = (gt.x as u32) << 0 | (gt.y as u32) << 1 | (gt.z as u32) << 2;
            idx = first_child + child_idx;
            let child_dir = gt * 2 - IVec3::ONE;
            center += IVec3::splat(size as i32 / 2) * child_dir;
        }
        // SVO now goes to desired depth, so we can mutate the node now.
        self.mut_node(idx).set_voxel(voxel);
        self.mut_node(idx).set_split_flag(false);
        Ok(result)
    }

    pub fn fill_voxels(&mut self, a: IVec3, b: IVec3, voxel: Voxel) {
        let min = ivec3(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
        let max = ivec3(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    drop(self.set_voxel(ivec3(x, y, z), voxel));
                }
            }
        }
    }

    pub fn surface_at(&self, x: i32, z: i32) -> Result<i32, WorldErr> {
        for y in 0..self.size as i32 {
            if self.get_voxel(ivec3(x, y, z))?.is_empty() {
                return Ok(y);
            }
        }
        Err(WorldErr::OutOfBounds)
    }

    pub fn get_collisions_w(&self, aabb: &Aabb) -> Vec<Aabb> {
        let mut aabbs = Vec::new();

        let from = aabb.from.floor().as_ivec3();
        let to = aabb.to.ceil().as_ivec3();

        for x in from.x..to.x {
            for y in from.y..to.y {
                for z in from.z..to.z {
                    let pos = ivec3(x, y, z);

                    let voxel = self.get_voxel(pos).unwrap_or(Voxel::AIR);

                    if !voxel.is_empty() {
                        let min = pos.as_vec3();
                        let max = min + 1.0;
                        aabbs.push(Aabb::new(min, max));
                    }
                }
            }
        }
        aabbs
    }

    pub fn sphere(&mut self, pos: IVec3, r: u32, voxel: Voxel, decay: f32) {
        let pos_center = pos.as_vec3() + Vec3::splat(0.5);
        let min = pos - IVec3::splat(r as i32);
        let max = pos + IVec3::splat(r as i32);
        let r_sq = r as f32 * r as f32;

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let block_center = ivec3(x, y, z).as_vec3() + Vec3::splat(0.5);
                    let dist_sq = (block_center - pos_center).length_squared();

                    if dist_sq >= r_sq || fastrand::f32() <= decay {
                        continue;
                    }

                    drop(self.set_voxel(ivec3(x, y, z), voxel));
                }
            }
        }
    }

    pub fn bounded_sphere(&mut self, pos: IVec3, r: u32, voxel: Voxel, min: IVec3, max: IVec3) {
        let pos_center = pos.as_vec3() + Vec3::splat(0.5);
        let r_sq = r as f32 * r as f32;

        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let block_center = ivec3(x, y, z).as_vec3() + Vec3::splat(0.5);
                    let dist_sq = (block_center - pos_center).length_squared();

                    if dist_sq >= r_sq {
                        continue;
                    }

                    drop(self.set_voxel(ivec3(x, y, z), voxel));
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct NoiseMaps {
    height: NoiseMap,
    freq: NoiseMap,
    scale: NoiseMap,
    bumps: NoiseMap,
    mountains: NoiseMap,
    temp: NoiseMap,
    moisture: NoiseMap,
    vegetation: NoiseMap,
}
impl NoiseMaps {
    pub fn from_seed(seed: i64) -> Self {
        Self {
            height: NoiseMap::new(seed.wrapping_mul(4326742), 0.003, 2.5),
            freq: NoiseMap::new(seed.wrapping_mul(927144), 0.0001, 7.0),
            scale: NoiseMap::new(seed.wrapping_mul(43265), 0.003, 40.0),
            bumps: NoiseMap::new(seed.wrapping_mul(76324), 0.15, 4.0),
            mountains: NoiseMap::new(seed.wrapping_mul(72316423), 0.001, 40.0),
            temp: NoiseMap::new(seed.wrapping_mul(83226), 0.0004, 1.0),
            moisture: NoiseMap::new(seed.wrapping_mul(2345632), 0.0004, 1.0),
            vegetation: NoiseMap::new(seed.wrapping_mul(53252), 0.001, 1.0),
        }
    }

    pub fn terrain_height(&self, x: f32, z: f32) -> f32 {
        let freq = self.freq.get(vec2(x, z));
        let scale = self.scale.get(vec2(x, z));
        self.height.get(vec2(x * freq, z * freq)) * scale
            + self.bumps.get(vec2(x, z))
            + self.mountains.get(vec2(x, z))
    }

    pub fn temp(&self, x: f32, z: f32) -> f32 {
        self.temp.get(vec2(x, z))
    }

    pub fn moisture(&self, x: f32, z: f32) -> f32 {
        self.moisture.get(vec2(x, z))
    }

    pub fn vegetation(&self, x: f32, z: f32) -> f32 {
        self.vegetation.get(vec2(x, z))
    }
}

pub struct WorldGen {
    pub maps: NoiseMaps,
    oak_tree_gen: TreeGen,
    birch_tree_gen: TreeGen,
}

impl WorldGen {
    pub fn new(seed: i64) -> Self {
        let oak_tree_gen = TreeGen {
            height: 6..19,
            bark: Voxel::OAK_WOOD,
            leaves: Voxel::OAK_LEAVES,
            leaves_decay: 0.1,
            branch_count: 1..4,
            branch_height: 0.5..0.8,
            branch_len: 3.0..8.0,
        };
        let birch_tree_gen = TreeGen {
            height: 9..26,
            bark: Voxel::BIRCH_WOOD,
            leaves: Voxel::BIRCH_LEAVES,
            leaves_decay: 0.1,
            branch_count: 1..4,
            branch_height: 0.5..0.8,
            branch_len: 3.0..8.0,
        };
        let maps = NoiseMaps::from_seed(seed);
        Self {
            maps,
            birch_tree_gen,
            oak_tree_gen,
        }
    }

    pub fn populate(&self, min: IVec3, max: IVec3, world: &mut World) {
        for x in min.x..max.x {
            for z in min.z..max.z {
                let y = self.maps.terrain_height(x as f32, z as f32) as i32;
                let surface_pos = ivec3(x, y, z);

                world.fill_voxels(ivec3(x, 0, z), ivec3(x, y - 4, z), Voxel::STONE);
                world.fill_voxels(ivec3(x, y - 3, z), ivec3(x, y - 1, z), Voxel::DIRT);

                if y < 26 {
                    drop(world.set_voxel(surface_pos, Voxel::SAND));
                    world.fill_voxels(surface_pos + IVec3::Y, ivec3(x, 26, z), Voxel::WATER);
                    continue;
                }

                let temp = self.maps.temp(x as f32, z as f32);
                let moisture = self.maps.moisture(x as f32, z as f32);
                let vegetation = self.maps.vegetation(x as f32, z as f32);

                let surface = match (moisture, temp) {
                    (m, t) if m < 0.3 && t > 0.7 => Voxel::SAND,
                    (m, t) if m < 0.3 && t < 0.3 => Voxel::DEAD_GRASS,
                    (m, t) if m > 0.3 && t < 0.3 => Voxel::SNOW,
                    (m, t) if m > 0.7 && t > 0.7 => Voxel::MOIST_GRASS,
                    _ => Voxel::GRASS,
                };

                drop(world.set_voxel(surface_pos, surface));

                if y < 26 {
                    world.fill_voxels(surface_pos + IVec3::Y, ivec3(x, 26, z), Voxel::WATER);
                    continue;
                }

                if surface == Voxel::GRASS && fastrand::f32() < 0.005 * vegetation {
                    match fastrand::u8(0..2) {
                        0 => spawn_tree(world, surface_pos, &self.oak_tree_gen),
                        1 => spawn_tree(world, surface_pos, &self.birch_tree_gen),
                        _ => unreachable!(),
                    }
                }
                if surface == Voxel::SAND && fastrand::f32() < 0.01 * vegetation {
                    spawn_cactus(world, surface_pos)
                }
                if surface == Voxel::SNOW && fastrand::f32() < 0.003 * vegetation {
                    spawn_spruce_tree(world, surface_pos);
                }
            }
        }
    }
}

fn spawn_cactus(world: &mut World, pos: IVec3) {
    let pos = pos + IVec3::Y;
    let height = fastrand::i32(2..7);
    let splits = if height > 3 { fastrand::u32(0..4) } else { 0 };

    world.fill_voxels(pos, pos + IVec3::Y * height, Voxel::CACTUS);
    for _ in 0..splits {
        let split_h = fastrand::i32(1..height);
        let split_len = fastrand::i32(1..4);
        let dir = rand_cardinal_dir();

        drop(world.set_voxel(pos + IVec3::Y * split_h + dir, Voxel::CACTUS));
        world.fill_voxels(
            pos + IVec3::Y * split_h + dir * 2,
            pos + IVec3::Y * (split_h + split_len) + dir * 2,
            Voxel::CACTUS,
        );
    }
}

fn spawn_spruce_tree(world: &mut World, pos: IVec3) {
    let offset = fastrand::i32(4..8);
    let height = offset + fastrand::i32(10..18);

    let mut y = height;
    let mut r: i32 = 1;
    while y > offset {
        let c = pos + IVec3::Y * y;
        let min = c - ivec3(r, 0, r);
        let max = c + ivec3(r, 0, r);
        world.bounded_sphere(c, r as u32, Voxel::SPRUCE_LEAVES, min, max);

        r += 1;
        y -= 2;
    }
    world.fill_voxels(pos, pos + IVec3::Y * (height - 1), Voxel::SPRUCE_WOOD);
}

struct TreeGen {
    pub(crate) height: Range<u32>,
    pub(crate) bark: Voxel,
    pub(crate) leaves: Voxel,
    pub(crate) leaves_decay: f32,
    pub(crate) branch_count: Range<u32>,
    pub(crate) branch_height: Range<f32>,
    pub(crate) branch_len: Range<f32>,
}

fn spawn_tree(world: &mut World, surface: IVec3, tree: &TreeGen) {
    let height = fastrand::u32(tree.height.clone());
    let randf32 = |range: Range<f32>| -> f32 {
        let size = range.end - range.start;
        fastrand::f32() * size + range.start
    };

    // only create branches if the tree is tall
    let branch_count = if height < 11 {
        0
    } else {
        fastrand::u32(tree.branch_count.clone())
    };

    world.sphere(
        surface + ivec3(0, height as i32, 0),
        5,
        tree.leaves,
        tree.leaves_decay,
    );

    for _ in 0..branch_count {
        let branch_h = (randf32(tree.branch_height.clone()) * height as f32) as u32;
        let branch_len = randf32(tree.branch_len.clone());

        let branch_dir = rand_hem_dir(Vec3::Y);
        let start = ivec3(surface.x, surface.y + branch_h as i32, surface.z);
        let end = (start.as_vec3() + branch_dir * branch_len).as_ivec3();

        world.sphere(end, 3, tree.leaves, tree.leaves_decay);

        let line = voxel_math::walk_line(start, end);
        for pos in line {
            drop(world.set_voxel(pos, tree.bark));
        }
    }

    for i in 0..height as i32 {
        drop(world.set_voxel(surface + ivec3(0, i, 0), tree.bark));
    }
}

pub fn rand_cardinal_dir() -> IVec3 {
    [
        ivec3(-1, 0, 0),
        ivec3(1, 0, 0),
        ivec3(0, 0, -1),
        ivec3(0, 0, 1),
    ][fastrand::usize(0..4)]
}

pub fn rand_dir() -> Vec3 {
    fn rand_norm() -> f32 {
        let theta = 2.0 * std::f32::consts::PI * fastrand::f32();
        let rho = (-2.0 * fastrand::f32().ln()).sqrt();
        rho * theta.cos()
    }

    let x = rand_norm();
    let y = rand_norm();
    let z = rand_norm();
    vec3(x, y, z).normalize()
}

pub fn rand_hem_dir(norm: Vec3) -> Vec3 {
    let dir = rand_dir();
    dir * norm.dot(dir).signum()
}
