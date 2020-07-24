use super::{CommandPool, Device, Instance};
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

pub(crate) struct Buffer {
    pub size: usize,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn new(
        instance: &Instance,
        device: &Device,
        size: usize,
        buffer_usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let buffer = unsafe {
            device.device.create_buffer(
                &vk::BufferCreateInfo {
                    size: size as _,
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

        Self {
            size,
            buffer,
            memory,
        }
    }

    pub fn write<T>(&self, device: &super::Device, data: &[T]) {
        let ptr = unsafe {
            device
                .device
                .map_memory(self.memory, 0, self.size as _, vk::MemoryMapFlags::empty())
        }
        .unwrap();
        unsafe { data.as_ptr().copy_to(ptr as _, data.len()) };
        unsafe { device.device.unmap_memory(self.memory) };
    }

    pub fn copy_to(&self, device: &super::Device, command_pool: &CommandPool, dst_buffer: &Buffer) {
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
                self.buffer,
                dst_buffer.buffer,
                &[vk::BufferCopy {
                    size: self.size as _,
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
    }

    pub fn destory(&self, device: &Device) {
        unsafe {
            device.device.destroy_buffer(self.buffer, None);
            device.device.free_memory(self.memory, None);
        }
    }
}
