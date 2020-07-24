use super::{CommandPool, Device, Instance, Vertex};
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

pub struct VertexBuffer {
    pub vertices: Vec<super::Vertex>,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl VertexBuffer {
    pub fn new(
        instance: &Instance,
        device: &Device,
        command_pool: &CommandPool,
        vertices: Vec<Vertex>,
    ) -> Self {
        let buffer_size = std::mem::size_of::<Vertex>() * vertices.len();
        let (staging_buffer, staging_memory) = Self::create_buffer(
            &instance,
            &device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        Self::write_to_buffer_memory(&device, &buffer_size, &staging_memory, &vertices);
        let (buffer, memory) = Self::create_buffer(
            &instance,
            &device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        Self::copy_buffer(device, command_pool, &staging_buffer, &buffer, buffer_size);

        unsafe { device.device.destroy_buffer(staging_buffer, None) };
        unsafe { device.device.free_memory(staging_memory, None) };

        Self {
            vertices,
            buffer,
            memory,
        }
    }

    pub fn destory(&self, device: &Device) {
        unsafe {
            device.device.destroy_buffer(self.buffer, None);
            device.device.free_memory(self.memory, None);
        }
    }

    fn create_buffer(
        instance: &Instance,
        device: &Device,
        buffer_size: usize,
        buffer_usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let buffer = unsafe {
            device.device.create_buffer(
                &vk::BufferCreateInfo {
                    size: buffer_size as _,
                    usage: buffer_usage,
                    sharing_mode: vk::SharingMode::EXCLUSIVE,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();
        let memory_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };

        let memory = unsafe {
            device.device.allocate_memory(
                &vk::MemoryAllocateInfo {
                    allocation_size: memory_requirements.size,
                    memory_type_index: instance
                        .instance
                        .get_physical_device_memory_properties(device.physical_device)
                        .memory_types
                        .iter()
                        .enumerate()
                        .find(|(i, memeory_type)| {
                            ((memory_requirements.memory_type_bits & (1 << i)) != 0)
                                && (memeory_type.property_flags & memory_properties
                                    == memory_properties)
                        })
                        .unwrap()
                        .0 as _,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();
        unsafe { device.device.bind_buffer_memory(buffer, memory, 0) }.unwrap();
        (buffer, memory)
    }

    fn write_to_buffer_memory(
        device: &super::Device,
        buffer_size: &usize,
        memory: &vk::DeviceMemory,
        vertices: &[super::Vertex],
    ) {
        let data = unsafe {
            device
                .device
                .map_memory(*memory, 0, *buffer_size as _, vk::MemoryMapFlags::empty())
        }
        .unwrap();
        unsafe { vertices.as_ptr().copy_to(data as _, vertices.len()) };
        unsafe { device.device.unmap_memory(*memory) };
    }

    fn copy_buffer(
        device: &super::Device,
        command_pool: &CommandPool,
        src_buffer: &vk::Buffer,
        dst_buffer: &vk::Buffer,
        copy_size: usize,
    ) {
        let command_buffer = unsafe {
            device
                .device
                .allocate_command_buffers(&vk::CommandBufferAllocateInfo {
                    level: vk::CommandBufferLevel::PRIMARY,
                    command_pool: command_pool.command_pool,
                    command_buffer_count: 1,
                    ..Default::default()
                })
        }
        .unwrap()
        .remove(0);
        unsafe {
            device.device.begin_command_buffer(
                command_buffer,
                &vk::CommandBufferBeginInfo {
                    flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                    ..Default::default()
                },
            )
        }
        .unwrap();
        unsafe {
            device.device.cmd_copy_buffer(
                command_buffer,
                *src_buffer,
                *dst_buffer,
                &[vk::BufferCopy {
                    size: copy_size as _,
                    ..Default::default()
                }],
            );
            device.device.end_command_buffer(command_buffer)
        }
        .unwrap();
        unsafe {
            device.device.queue_submit(
                device.queue,
                &[vk::SubmitInfo {
                    command_buffer_count: 1,
                    p_command_buffers: &command_buffer,
                    ..Default::default()
                }],
                vk::Fence::null(),
            )
        }
        .unwrap();
        unsafe { device.device.queue_wait_idle(device.queue) }.unwrap();
        unsafe {
            device
                .device
                .free_command_buffers(command_pool.command_pool, &[command_buffer])
        };
        /*
        vkFreeCommandBuffers(device, commandPool, 1, &commandBuffer); */
    }
}
