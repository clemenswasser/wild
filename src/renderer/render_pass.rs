use ash::vk;

pub struct RenderPass {
    pub render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(surface: &super::Surface, device: &super::Device) -> Self {
        Self {
            render_pass: unsafe {
                device.device.create_render_pass(
                    &vk::RenderPassCreateInfo {
                        attachment_count: 1,
                        p_attachments: &vk::AttachmentDescription {
                            format: surface.format.unwrap().format,
                            samples: vk::SampleCountFlags::TYPE_1,
                            load_op: vk::AttachmentLoadOp::CLEAR,
                            store_op: vk::AttachmentStoreOp::STORE,
                            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                            initial_layout: vk::ImageLayout::UNDEFINED,
                            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                            ..Default::default()
                        },
                        subpass_count: 1,
                        p_subpasses: &vk::SubpassDescription {
                            color_attachment_count: 1,
                            p_color_attachments: &vk::AttachmentReference {
                                attachment: 0,
                                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                            },
                            ..Default::default()
                        },
                        dependency_count: 1,
                        p_dependencies: &vk::SubpassDependency {
                            src_subpass: vk::SUBPASS_EXTERNAL,
                            dst_subpass: 0,
                            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                )
            }
            .unwrap(),
        }
    }

    pub fn destroy(&self, device: &super::Device) {
        unsafe { device.device.destroy_render_pass(self.render_pass, None) };
    }
}
