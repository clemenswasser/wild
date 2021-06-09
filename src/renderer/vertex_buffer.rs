use super::{Buffer, CommandPool, Device, Instance, Vertex};
use ash::vk;

pub struct VertexBuffer {
    pub vertices: Vec<Vertex>,
    pub buffer: Buffer,
}

impl VertexBuffer {
    pub fn new(
        instance: &Instance,
        device: &Device,
        command_pool: &CommandPool,
        vertices: Vec<Vertex>,
    ) -> Self {
        let buffer_size = std::mem::size_of::<Vertex>() * vertices.len();
        let staging_buffer = Buffer::new(
            instance,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        staging_buffer.write_arr(device, &vertices);
        let buffer = Buffer::new(
            instance,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        staging_buffer.copy_to(device, command_pool, &buffer);
        staging_buffer.destory(device);

        Self { vertices, buffer }
    }

    pub fn destory(&self, device: &Device) {
        self.buffer.destory(device);
    }
}
