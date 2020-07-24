use ash::version::DeviceV1_0;
use ash::vk;

pub(crate) struct Swapchain {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn new(
        instance: &super::Instance,
        surface: &super::Surface,
        device: &super::Device,
    ) -> Self {
        let loader = ash::extensions::khr::Swapchain::new(&instance.instance, &device.device);
        let swapchain = unsafe {
            loader.create_swapchain(
                &vk::SwapchainCreateInfoKHR {
                    surface: surface.surface,
                    min_image_count: surface.capabilities.unwrap().min_image_count,
                    image_format: surface.format.unwrap().format,
                    image_color_space: surface.format.unwrap().color_space,
                    image_extent: surface.capabilities.unwrap().current_extent,
                    image_array_layers: 1,
                    image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                    image_sharing_mode: vk::SharingMode::EXCLUSIVE,
                    pre_transform: surface.capabilities.unwrap().current_transform,
                    composite_alpha: surface.capabilities.unwrap().supported_composite_alpha,
                    present_mode: vk::PresentModeKHR::MAILBOX,
                    clipped: vk::TRUE,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();
        let images = unsafe { loader.get_swapchain_images(swapchain) }.unwrap();
        let image_views = images
            .iter()
            .map(|image| {
                unsafe {
                    device.device.create_image_view(
                        &vk::ImageViewCreateInfo {
                            image: *image,
                            view_type: vk::ImageViewType::TYPE_2D,
                            format: surface.format.unwrap().format,
                            components: vk::ComponentMapping {
                                r: vk::ComponentSwizzle::IDENTITY,
                                g: vk::ComponentSwizzle::IDENTITY,
                                b: vk::ComponentSwizzle::IDENTITY,
                                a: vk::ComponentSwizzle::IDENTITY,
                            },
                            subresource_range: vk::ImageSubresourceRange {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                base_mip_level: 0,
                                level_count: 1,
                                base_array_layer: 0,
                                layer_count: 1,
                            },
                            ..Default::default()
                        },
                        None,
                    )
                }
                .unwrap()
            })
            .collect::<Vec<_>>();

        Self {
            loader,
            swapchain,
            images,
            image_views,
        }
    }

    pub fn destroy(&self, device: &super::Device) {
        self.image_views
            .iter()
            .for_each(|image_view| unsafe { device.device.destroy_image_view(*image_view, None) });
        unsafe { self.loader.destroy_swapchain(self.swapchain, None) };
    }
}
