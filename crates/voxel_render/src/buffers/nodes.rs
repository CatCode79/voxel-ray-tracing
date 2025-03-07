//= IMPORTS ==================================================================

use crate::Voxel;

use voxel_math::BitField;

use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

use std::slice;

//= NODE =====================================================================

/// Represents a node in the sparse voxel octree (SVO) that is the world.
///
/// ## Underlying Implementation
/// There are a lot of nodes in a world,
/// so I've tried to make them use as little memory as I could.
/// Each node consumes 4 bytes of memory, a single 32-bit integer.
/// Here are the different states of the bits:
///
/// ```
/// 00______________________________
/// ```
/// Node is not used.
///
/// ```
/// 10______________________________
/// ```
/// Invalid state.
///
/// ```
/// 01______________________xxxxxxxx
/// ```
/// Node is a single voxel where x = voxel type.
///
/// ```
/// 11xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
/// ```
/// Node is split into 8 nodes of half size where x points to first child.
/// All 8 child nodes will be sequential in memory so only the position of the first one is needed.
/// NOTE: the index of the first child will always be one more than a multiple of 8,
/// so x actually represents `(child_index - 1) / 8`.
///
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Node(BitField);
impl Node {
    pub const ZERO: Self = Self(BitField::ZERO);

    pub fn new_leaf(voxel: Voxel) -> Self {
        let mut rs = Self::ZERO;
        rs.set_voxel(voxel);
        rs.set_used_flag(true);
        rs
    }

    pub fn new_split(first_child: u32) -> Self {
        let mut rs = Self::ZERO;
        rs.set_first_child(first_child);
        rs.set_split_flag(true);
        rs.set_used_flag(true);
        rs
    }

    #[inline(always)]
    pub fn set_used_flag(&mut self, f: bool) {
        self.0.set(f as u32, 1, 30)
    }
    #[inline(always)]
    pub fn is_used(self) -> bool {
        self.0.get(1, 30) == 1
    }

    #[inline(always)]
    pub fn set_split_flag(&mut self, f: bool) {
        self.0.set(f as u32, 1, 31)
    }
    #[inline(always)]
    pub fn is_split(self) -> bool {
        self.0.get(1, 31) == 1
    }

    #[inline(always)]
    pub fn get_voxel(self) -> Voxel {
        Voxel(self.0.get(8, 0) as u8)
    }
    #[inline(always)]
    pub fn set_voxel(&mut self, voxel: Voxel) {
        self.0.set(voxel.0 as u32, 8, 0)
    }

    #[inline(always)]
    pub fn set_first_child(&mut self, first_child: u32) {
        debug_assert!(first_child == 0 || ((first_child - 1) % 8) == 0);
        let first_child = (first_child - 1) / 8;

        self.0.set(first_child, 30, 0)
    }
    #[inline(always)]
    pub fn first_child(self) -> u32 {
        self.0.get(30, 0) * 8 + 1
    }

    #[inline(always)]
    pub fn get_child(self, idx: u32) -> u32 {
        self.first_child() + idx
    }

    pub fn split(&mut self, first_child: u32) {
        self.set_split_flag(true);
        self.set_first_child(first_child);
    }

    /// Call if `Self::can_simplify` returns `true`.
    pub fn simplify(&mut self, result: Voxel) {
        self.set_split_flag(false);
        self.set_voxel(result);
    }
}

//= NODES BUFFER =============================================================

pub struct NodesBuffer {
    pub buf: Buffer,
    pub count: u32,
}

impl NodesBuffer {
    pub fn new(device: &Device, label: &str, usage: BufferUsages, count: u32) -> Self {
        let buf = device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: count as u64 * size_of::<Node>() as u64,
            usage,
            mapped_at_creation: false,
        });
        Self { buf, count }
    }

    pub fn write(&self, queue: &Queue, offset: u64, nodes: &[Node]) {
        let nodes_cut = (nodes.len() as u64).min(self.count as u64 - offset);
        let nodes: &[Node] = &nodes[0..nodes_cut as usize];

        let ptr = nodes.as_ptr() as *const u8;
        let size = nodes.len() * size_of::<Node>();
        #[allow(unsafe_code)]
        let slice = unsafe { slice::from_raw_parts(ptr, size) };
        let offset = offset * size_of::<Node>() as u64;
        queue.write_buffer(&self.buf, offset, slice);
    }
}
