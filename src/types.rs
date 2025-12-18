
use ash::vk;

pub trait AttributeDescriptions {
    fn attr_desc() -> Vec<vk::VertexInputAttributeDescription>;
}

pub trait BindingDescriptions {
    fn bind_desc() -> Vec<vk::VertexInputBindingDescription>;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl AttributeDescriptions for Vertex {
    fn attr_desc() -> Vec<vk::VertexInputAttributeDescription> {
        let attributes = vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: std::mem::offset_of!(Vertex, color) as u32,
            },
        ];

        attributes
    }
}

impl BindingDescriptions for Vertex {
    fn bind_desc() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }
}