use std::collections::HashMap;

use wgpu::{util::DeviceExt, BindGroupLayout, Buffer, Device, IndexFormat, ShaderStages};

use crate::vertex::{INDICES, VERTICES};

pub struct Buffers {
    pub vertex: VertexBuffer,
    pub index: IndexBuffer,
    pub bind_groups: Vec<BindGroup>,
}

impl Buffers {
    pub fn new(device: &Device) -> Self {
        let vertex = VertexBuffer::new(device, VERTICES);
        let index = IndexBuffer::new(device, INDICES);
        let bind_groups = vec![];
        Self {
            vertex,
            index,
            bind_groups,
        }
    }
    pub fn bind_group<T: bytemuck::Pod>(mut self, bind_group: BindGroup) -> Self {
        self.bind_groups.push(bind_group);
        self
    }
}

pub struct VertexBuffer {
    pub buffer: Buffer,
}

impl VertexBuffer {
    pub fn new<T: bytemuck::Pod>(device: &Device, vertices: &[T]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self { buffer }
    }
}

trait FormatDetect {
    fn to_index_format() -> IndexFormat;
}
impl FormatDetect for u32 {
    fn to_index_format() -> IndexFormat {
        IndexFormat::Uint32
    }
}
impl FormatDetect for u16 {
    fn to_index_format() -> IndexFormat {
        IndexFormat::Uint16
    }
}
pub struct IndexBuffer {
    pub buffer: Buffer,
    pub format: IndexFormat,
}
impl IndexBuffer {
    #[allow(private_bounds)]
    pub fn new<T: bytemuck::Pod + FormatDetect>(device: &Device, indices: &[T]) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            buffer,
            format: T::to_index_format(),
        }
    }
}

struct BindGroupLayoutEntry {
    /// Which shader stages can see this binding.
    pub visibility: ShaderStages,
    /// The type of the binding
    pub ty: wgpu::BindingType,
    /// If this value is Some, indicates this entry is an array. Array size must be 1 or greater.
    ///
    /// If this value is Some and `ty` is `BindingType::Texture`, [`Features::TEXTURE_BINDING_ARRAY`] must be supported.
    ///
    /// If this value is Some and `ty` is any other variant, bind group creation will fail.
    #[cfg_attr(any(feature = "trace", feature = "replay"), serde(default))]
    pub count: Option<std::num::NonZeroU32>,
}
impl BindGroupLayoutEntry {
    fn to_wgpu(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: self.ty,
            #[cfg_attr(any(feature = "trace", feature = "replay"), serde(default))]
            count: self.count,
        }
    }
}
pub struct BindGroupEntry {
    buffer: wgpu::Buffer,
    layout: BindGroupLayoutEntry,
}

pub struct BindGroup {
    label: String,
    layout: wgpu::BindGroupLayout,

    entries: Vec<BindGroupEntry>,
    entry_labels: Vec<String>,
    entry_layout: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroup {
    pub fn new(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),

            entries: vec![],
            entry_labels: vec![],
            entry_layout: vec![],
        }
    }
    pub fn insert(mut self, label: String, entry: BindGroupEntry) -> Self {
        self.entries.push(entry);
        self.entry_labels.push(label);
        self.entry_layout
            .push(entry.layout.to_wgpu(self.entries.len() as u32));
        self
    }

    fn bind_group_layouts(&self, device: &Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &self.entry_layout,
            label: Some(&format!("{}_layout", self.label)),
        })
    }
    fn bind_group(&self, device: &Device) -> wgpu::BindGroup {
        let entries = self
            .entries
            .iter()
            .enumerate()
            .map(|(binding, entry)| wgpu::BindGroupEntry {
                binding: binding as u32,
                resource: entry.buffer.as_entire_binding(),
            })
            .collect::<Vec<_>>();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &entries,
            label: Some(&self.label),
        })
    }
}
