use ash::vk;

pub struct CommandPool {
    pub command_pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: &super::Device) -> Self {
        let command_pool = unsafe {
            device.device.create_command_pool(
                &vk::CommandPoolCreateInfo {
                    queue_family_index: 0,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        Self { command_pool }
    }

    pub fn destroy(&self, device: &super::Device) {
        unsafe { device.device.destroy_command_pool(self.command_pool, None) };
    }
}
