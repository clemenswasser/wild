use ash::vk;

#[repr(C)]
pub struct Vertex {
    position: cgmath::Vector2<f32>,
    color: cgmath::Vector3<f32>,
}

impl Vertex {
    pub fn new(position: cgmath::Vector2<f32>, color: cgmath::Vector3<f32>) -> Self {
        Self { position, color }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Self>() as _,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: memoffset::offset_of!(Vertex, position) as _,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: memoffset::offset_of!(Vertex, color) as _,
            },
        ]
    }
}
