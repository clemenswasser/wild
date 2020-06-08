use ash::version::DeviceV1_0;
use ash::vk;

pub struct CommandPool {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
}

impl CommandPool {
    pub fn new(
        surface: &super::Surface,
        device: &super::Device,
        render_pass: &super::RenderPass,
        pipeline: &super::Pipeline,
        framebuffers: &super::Framebuffers,
    ) -> Self {
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

        let command_buffers = unsafe {
            device
                .device
                .allocate_command_buffers(&vk::CommandBufferAllocateInfo {
                    command_pool,
                    level: vk::CommandBufferLevel::PRIMARY,
                    command_buffer_count: framebuffers.framebuffers.len() as _,
                    ..Default::default()
                })
        }
        .unwrap();

        command_buffers
            .iter()
            .enumerate()
            .for_each(|(i, command_buffer)| {
                unsafe {
                    device
                        .device
                        .begin_command_buffer(
                            *command_buffer,
                            &vk::CommandBufferBeginInfo::default(),
                        )
                        .unwrap();
                    device.device.cmd_begin_render_pass(
                        *command_buffer,
                        &vk::RenderPassBeginInfo {
                            render_pass: render_pass.render_pass,
                            framebuffer: *framebuffers.framebuffers.get(i).unwrap(),
                            render_area: vk::Rect2D {
                                offset: vk::Offset2D { x: 0, y: 0 },
                                extent: surface.capabilities.unwrap().current_extent,
                            },
                            clear_value_count: 1,
                            p_clear_values: &vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [0.0, 0.0, 0.0, 1.0],
                                },
                            },
                            ..Default::default()
                        },
                        vk::SubpassContents::INLINE,
                    );
                    device.device.cmd_bind_pipeline(
                        *command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        pipeline.pipeline,
                    );
                    device.device.cmd_draw(*command_buffer, 3, 1, 0, 0);
                    device.device.cmd_end_render_pass(*command_buffer);
                    device.device.end_command_buffer(*command_buffer).unwrap();
                };
            });

        Self {
            command_pool,
            command_buffers,
        }
    }

    pub fn destroy(&self, device: &super::Device) {
        unsafe {
            device
                .device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            device.device.destroy_command_pool(self.command_pool, None);
        }
    }
}
