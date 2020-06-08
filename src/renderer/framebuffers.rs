use ash::version::DeviceV1_0;
use ash::vk;

pub struct Framebuffers {
    pub framebuffers: Vec<vk::Framebuffer>,
}

impl Framebuffers {
    pub fn new(
        surface: &super::Surface,
        device: &super::Device,
        swapchain: &super::Swapchain,
        render_pass: &super::RenderPass,
    ) -> Self {
        Self {
            framebuffers: swapchain
                .image_views
                .iter()
                .map(|image_view| {
                    unsafe {
                        device.device.create_framebuffer(
                            &vk::FramebufferCreateInfo {
                                render_pass: render_pass.render_pass,
                                attachment_count: 1,
                                p_attachments: image_view,
                                width: surface.capabilities.unwrap().current_extent.width,
                                height: surface.capabilities.unwrap().current_extent.height,
                                layers: 1,
                                ..Default::default()
                            },
                            None,
                        )
                    }
                    .unwrap()
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn destroy(&self, device: &super::Device) {
        self.framebuffers.iter().for_each(|framebuffer| unsafe {
            device.device.destroy_framebuffer(*framebuffer, None)
        });
    }
}
