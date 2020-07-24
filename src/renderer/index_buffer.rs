use super::{Buffer, CommandPool, Device, Instance};
use ash::vk;

pub struct IndexBuffer {
    pub indices: Vec<u16>,
    pub buffer: Buffer,
}

impl IndexBuffer {
    pub fn new(
        instance: &Instance,
        device: &Device,
        command_pool: &CommandPool,
        indices: Vec<u16>,
    ) -> Self {
        let buffer_size = std::mem::size_of::<u16>() * indices.len();
        let staging_buffer = Buffer::new(
            instance,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        staging_buffer.write(device, &indices);
        let buffer = Buffer::new(
            instance,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        staging_buffer.copy_to(device, command_pool, &buffer);
        staging_buffer.destory(device);
        Self { indices, buffer }
    }

    pub fn destory(&self, device: &Device) {
        self.buffer.destory(device);
    }
}
