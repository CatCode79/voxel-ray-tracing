//= MODULES ==================================================================

mod camera;
mod frame;
mod nodes;
mod settings;
mod voxel;
mod world;

//= RE-EXPORTS ===============================================================

pub use camera::*;
pub(crate) use frame::*;
pub use nodes::*;
pub use settings::*;
pub use voxel::*;
pub use world::*;

//= IMPORTS ==================================================================

use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

use std::slice;

//= SIMPLE BUFFER ============================================================

pub(crate) struct SimpleBuffer<T>(pub(crate) Buffer, std::marker::PhantomData<T>);

impl<T> SimpleBuffer<T> {
    pub(crate) fn new(device: &Device, label: &str, usage: BufferUsages) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size: size_of::<T>() as u64,
            usage,
            mapped_at_creation: false,
        });
        Self(buffer, std::marker::PhantomData)
    }

    pub(crate) fn write(&self, queue: &Queue, value: &T) {
        let coerced: *const T = value;
        let ptr = coerced as *const u8;
        let size = size_of::<T>();
        #[allow(unsafe_code)]
        let slice = unsafe { slice::from_raw_parts(ptr, size) };
        queue.write_buffer(&self.0, 0, slice);
    }
}

impl<T, const N: usize> SimpleBuffer<[T; N]> {
    pub(crate) fn write_slice(&self, queue: &Queue, idx: u64, value: &[T]) {
        let ptr = value.as_ptr() as *const u8;
        let size = size_of::<T>() * value.len();
        #[allow(unsafe_code)]
        let slice = unsafe { slice::from_raw_parts(ptr, size) };
        queue.write_buffer(&self.0, idx, slice);
    }
}

//= BUFFERS ==================================================================

pub(crate) struct Buffers {
    pub(crate) camera_buffer: SimpleBuffer<Camera>,
    pub(crate) settings: SimpleBuffer<Settings>,
    pub(crate) world_data: SimpleBuffer<WorldData>,
    pub(crate) nodes: NodesBuffer,
    pub(crate) voxel_materials: SimpleBuffer<[Material; 256]>,
    pub(crate) frame_count: SimpleBuffer<FrameData>,
}

impl Buffers {
    pub(crate) fn new(device: &Device, max_nodes: u32) -> Self {
        const COPY_DST: BufferUsages = BufferUsages::COPY_DST;
        const UNIFORM: BufferUsages = BufferUsages::UNIFORM;
        const STORAGE: BufferUsages = BufferUsages::STORAGE;

        Self {
            camera_buffer: SimpleBuffer::new(device, "", COPY_DST | UNIFORM),
            settings: SimpleBuffer::new(device, "", COPY_DST | UNIFORM),
            world_data: SimpleBuffer::new(device, "", COPY_DST | UNIFORM),
            nodes: NodesBuffer::new(device, "", COPY_DST | STORAGE, max_nodes),
            voxel_materials: SimpleBuffer::new(device, "", COPY_DST | STORAGE),
            frame_count: SimpleBuffer::new(device, "", COPY_DST | UNIFORM),
        }
    }
}
