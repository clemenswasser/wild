mod device;
mod entry;
mod instance;
mod surface;

use device::Device;
use entry::Entry;
use instance::Instance;
use surface::Surface;

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct SyncObjects {
    image_availabe_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT],
    images_in_flight: Vec<Option<vk::Fence>>,
}

pub struct Renderer {
    pub entry: Entry,
    pub instance: Instance,
    pub surface: Surface,
    pub device: Device,
    pub queue: vk::Queue,
    pub swapchain: (ash::extensions::khr::Swapchain, vk::SwapchainKHR),
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub sync_objects: SyncObjects,
    pub current_frame: usize,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = Entry::new();
        let instance = Instance::new(&entry, window);
        let mut surface = Surface::new(window, &entry, &instance);
        let device = Device::new(&instance, &mut surface);
        let queue = unsafe { device.device.get_device_queue(0, 0) };

        let (swapchain_loader, swapchain) = Self::create_swapchain(
            &instance.instance,
            &device.device,
            &surface.surface,
            &surface.capabilities.unwrap(),
            &surface.format.unwrap(),
        );

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.unwrap();

        let swapchain_image_views = Self::create_swapchain_image_views(
            &swapchain_images,
            &device.device,
            &surface.format.unwrap(),
        );

        let render_pass = Self::create_renderpass(&device.device, &surface.format.unwrap());

        let (vertex_shader_module, fragment_shader_module) =
            Self::create_shader_modules(&device.device);

        let pipeline_layout = unsafe {
            device.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo {
                    set_layout_count: 0,
                    push_constant_range_count: 0,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        let pipeline = Self::create_pipeline(
            &device.device,
            &render_pass,
            &surface.capabilities.unwrap(),
            &(vertex_shader_module, fragment_shader_module),
            &pipeline_layout,
        );

        unsafe {
            device
                .device
                .destroy_shader_module(vertex_shader_module, None)
        };
        unsafe {
            device
                .device
                .destroy_shader_module(fragment_shader_module, None)
        };

        let framebuffers = Self::create_framebuffers(
            &device.device,
            &swapchain_image_views,
            &render_pass,
            &surface.capabilities.unwrap(),
        );

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

        let command_buffers = Self::create_command_buffers(
            &device.device,
            &surface.capabilities.unwrap(),
            &render_pass,
            &pipeline,
            &framebuffers,
            &command_pool,
        );

        let sync_objects = Self::create_sync_objects(&device.device, swapchain_images.len());

        Self {
            entry,
            instance,
            surface,
            device,
            queue,
            swapchain: (swapchain_loader, swapchain),
            swapchain_images,
            swapchain_image_views,
            render_pass,
            pipeline_layout,
            pipeline,
            framebuffers,
            command_pool,
            command_buffers,
            sync_objects,
            current_frame: 0,
        }
    }

    pub fn render(&mut self) {
        unsafe {
            self.device.device.wait_for_fences(
                &[*self
                    .sync_objects
                    .in_flight_fences
                    .get(self.current_frame)
                    .unwrap()],
                true,
                std::u64::MAX,
            )
        }
        .unwrap();

        let (image_index, _) = unsafe {
            self.swapchain.0.acquire_next_image(
                self.swapchain.1,
                std::u64::MAX,
                *self
                    .sync_objects
                    .image_availabe_semaphores
                    .get(self.current_frame)
                    .unwrap(),
                vk::Fence::null(),
            )
        }
        .unwrap();

        if let Some(images_in_flight_fence) = self
            .sync_objects
            .images_in_flight
            .get_mut(image_index as usize)
            .unwrap()
        {
            unsafe {
                self.device
                    .device
                    .wait_for_fences(&[*images_in_flight_fence], true, std::u64::MAX)
            }
            .unwrap();
            *images_in_flight_fence = *self
                .sync_objects
                .in_flight_fences
                .get(self.current_frame)
                .unwrap();
        }

        unsafe {
            self.device.device.reset_fences(&[*self
                .sync_objects
                .in_flight_fences
                .get(self.current_frame)
                .unwrap()])
        }
        .unwrap();

        unsafe {
            self.device.device.queue_submit(
                self.queue,
                &[vk::SubmitInfo {
                    wait_semaphore_count: 1,
                    p_wait_semaphores: self
                        .sync_objects
                        .image_availabe_semaphores
                        .get(self.current_frame)
                        .unwrap(),
                    p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    command_buffer_count: 1,
                    p_command_buffers: self.command_buffers.get(image_index as usize).unwrap(),
                    signal_semaphore_count: 1,
                    p_signal_semaphores: self
                        .sync_objects
                        .render_finished_semaphores
                        .get(self.current_frame)
                        .unwrap(),
                    ..Default::default()
                }],
                *self
                    .sync_objects
                    .in_flight_fences
                    .get(self.current_frame)
                    .unwrap(),
            )
        }
        .unwrap();

        unsafe {
            self.swapchain.0.queue_present(
                self.queue,
                &vk::PresentInfoKHR {
                    wait_semaphore_count: 1,
                    p_wait_semaphores: self
                        .sync_objects
                        .render_finished_semaphores
                        .get(self.current_frame)
                        .unwrap(),
                    swapchain_count: 1,
                    p_swapchains: &self.swapchain.1,
                    p_image_indices: &image_index,
                    ..Default::default()
                },
            )
        }
        .unwrap();
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        surface: &vk::SurfaceKHR,
        capabilities: &vk::SurfaceCapabilitiesKHR,
        surface_format: &vk::SurfaceFormatKHR,
    ) -> (ash::extensions::khr::Swapchain, vk::SwapchainKHR) {
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(
                &vk::SwapchainCreateInfoKHR {
                    surface: *surface,
                    min_image_count: capabilities.min_image_count,
                    image_format: surface_format.format,
                    image_color_space: surface_format.color_space,
                    image_extent: capabilities.current_extent,
                    image_array_layers: 1,
                    image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                    image_sharing_mode: vk::SharingMode::EXCLUSIVE,
                    pre_transform: capabilities.current_transform,
                    composite_alpha: capabilities.supported_composite_alpha,
                    present_mode: vk::PresentModeKHR::MAILBOX,
                    clipped: vk::TRUE,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        (swapchain_loader, swapchain)
    }

    fn create_swapchain_image_views(
        swapchain_images: &[vk::Image],
        device: &ash::Device,
        surface_format: &vk::SurfaceFormatKHR,
    ) -> Vec<vk::ImageView> {
        swapchain_images
            .iter()
            .map(|image| {
                unsafe {
                    device.create_image_view(
                        &vk::ImageViewCreateInfo {
                            image: *image,
                            view_type: vk::ImageViewType::TYPE_2D,
                            format: surface_format.format,
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
            .collect::<Vec<_>>()
    }

    fn create_renderpass(
        device: &ash::Device,
        surface_format: &vk::SurfaceFormatKHR,
    ) -> vk::RenderPass {
        unsafe {
            device.create_render_pass(
                &vk::RenderPassCreateInfo {
                    attachment_count: 1,
                    p_attachments: &vk::AttachmentDescription {
                        format: surface_format.format,
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
        .unwrap()
    }

    fn create_shader_modules(device: &ash::Device) -> (vk::ShaderModule, vk::ShaderModule) {
        let vertex_shader = include_bytes!("../../shaders/vert.spv");
        let fragment_shader = include_bytes!("../../shaders/frag.spv");

        (
            unsafe {
                device.create_shader_module(
                    #[allow(clippy::cast_ptr_alignment)]
                    &vk::ShaderModuleCreateInfo {
                        code_size: vertex_shader.len(),
                        p_code: vertex_shader.as_ptr() as _,
                        ..Default::default()
                    },
                    None,
                )
            }
            .unwrap(),
            unsafe {
                device.create_shader_module(
                    #[allow(clippy::cast_ptr_alignment)]
                    &vk::ShaderModuleCreateInfo {
                        code_size: fragment_shader.len(),
                        p_code: fragment_shader.as_ptr() as _,
                        ..Default::default()
                    },
                    None,
                )
            }
            .unwrap(),
        )
    }

    fn create_pipeline(
        device: &ash::Device,
        render_pass: &vk::RenderPass,
        capabilities: &vk::SurfaceCapabilitiesKHR,
        (vertex_shader_module, fragment_shader_module): &(vk::ShaderModule, vk::ShaderModule),
        pipeline_layout: &vk::PipelineLayout,
    ) -> vk::Pipeline {
        unsafe {
            device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[vk::GraphicsPipelineCreateInfo {
                    stage_count: 2,
                    p_stages: [
                        vk::PipelineShaderStageCreateInfo {
                            stage: vk::ShaderStageFlags::VERTEX,
                            module: *vertex_shader_module,
                            p_name: b"main\0".as_ptr() as _,
                            ..Default::default()
                        },
                        vk::PipelineShaderStageCreateInfo {
                            stage: vk::ShaderStageFlags::FRAGMENT,
                            module: *fragment_shader_module,
                            p_name: b"main\0".as_ptr() as _,
                            ..Default::default()
                        },
                    ]
                    .as_ptr(),
                    p_vertex_input_state: &vk::PipelineVertexInputStateCreateInfo {
                        vertex_binding_description_count: 0,
                        vertex_attribute_description_count: 0,
                        ..Default::default()
                    },
                    p_input_assembly_state: &vk::PipelineInputAssemblyStateCreateInfo {
                        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                        primitive_restart_enable: vk::FALSE,
                        ..Default::default()
                    },
                    p_viewport_state: &vk::PipelineViewportStateCreateInfo {
                        viewport_count: 1,
                        p_viewports: &vk::Viewport {
                            x: 0.0,
                            y: 0.0,
                            width: capabilities.current_extent.width as _,
                            height: capabilities.current_extent.height as _,
                            min_depth: 0.0,
                            max_depth: 1.0,
                        },
                        scissor_count: 1,
                        p_scissors: &vk::Rect2D {
                            offset: vk::Offset2D { x: 0, y: 0 },
                            extent: capabilities.current_extent,
                        },
                        ..Default::default()
                    },
                    p_rasterization_state: &vk::PipelineRasterizationStateCreateInfo {
                        depth_clamp_enable: vk::FALSE,
                        rasterizer_discard_enable: vk::FALSE,
                        polygon_mode: vk::PolygonMode::FILL,
                        line_width: 1.0,
                        cull_mode: vk::CullModeFlags::BACK,
                        front_face: vk::FrontFace::CLOCKWISE,
                        depth_bias_enable: vk::FALSE,
                        ..Default::default()
                    },
                    p_multisample_state: &vk::PipelineMultisampleStateCreateInfo {
                        sample_shading_enable: vk::FALSE,
                        rasterization_samples: vk::SampleCountFlags::TYPE_1,
                        ..Default::default()
                    },
                    p_color_blend_state: &vk::PipelineColorBlendStateCreateInfo {
                        logic_op_enable: vk::FALSE,
                        logic_op: vk::LogicOp::COPY,
                        attachment_count: 1,
                        p_attachments: &vk::PipelineColorBlendAttachmentState {
                            color_write_mask: vk::ColorComponentFlags::R
                                | vk::ColorComponentFlags::G
                                | vk::ColorComponentFlags::B
                                | vk::ColorComponentFlags::A,
                            blend_enable: vk::FALSE,
                            ..Default::default()
                        },
                        blend_constants: [0.0, 0.0, 0.0, 0.0],
                        ..Default::default()
                    },
                    layout: *pipeline_layout,
                    render_pass: *render_pass,
                    subpass: 0,
                    ..Default::default()
                }],
                None,
            )
        }
        .unwrap()
        .remove(0)
    }

    fn create_framebuffers(
        device: &ash::Device,
        swapchain_image_views: &[vk::ImageView],
        render_pass: &vk::RenderPass,
        capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> Vec<vk::Framebuffer> {
        swapchain_image_views
            .iter()
            .map(|image_view| {
                unsafe {
                    device.create_framebuffer(
                        &vk::FramebufferCreateInfo {
                            render_pass: *render_pass,
                            attachment_count: 1,
                            p_attachments: image_view,
                            width: capabilities.current_extent.width,
                            height: capabilities.current_extent.height,
                            layers: 1,
                            ..Default::default()
                        },
                        None,
                    )
                }
                .unwrap()
            })
            .collect::<Vec<_>>()
    }

    fn create_command_buffers(
        device: &ash::Device,
        capabilities: &vk::SurfaceCapabilitiesKHR,
        render_pass: &vk::RenderPass,
        pipeline: &vk::Pipeline,
        framebuffers: &[vk::Framebuffer],
        command_pool: &vk::CommandPool,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffers = unsafe {
            device.allocate_command_buffers(&vk::CommandBufferAllocateInfo {
                command_pool: *command_pool,
                level: vk::CommandBufferLevel::PRIMARY,
                command_buffer_count: framebuffers.len() as _,
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
                        .begin_command_buffer(
                            *command_buffer,
                            &vk::CommandBufferBeginInfo::default(),
                        )
                        .unwrap();
                    device.cmd_begin_render_pass(
                        *command_buffer,
                        &vk::RenderPassBeginInfo {
                            render_pass: *render_pass,
                            framebuffer: *framebuffers.get(i).unwrap(),
                            render_area: vk::Rect2D {
                                offset: vk::Offset2D { x: 0, y: 0 },
                                extent: capabilities.current_extent,
                            },
                            clear_value_count: 1,
                            p_clear_values: &vk::ClearValue {
                                color: vk::ClearColorValue {
                                    float32: [1.0, 1.0, 1.0, 1.0],
                                },
                            },
                            ..Default::default()
                        },
                        vk::SubpassContents::INLINE,
                    );
                    device.cmd_bind_pipeline(
                        *command_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        *pipeline,
                    );
                    device.cmd_draw(*command_buffer, 3, 1, 0, 0);
                    device.cmd_end_render_pass(*command_buffer);
                    device.end_command_buffer(*command_buffer).unwrap();
                };
            });
        command_buffers
    }

    fn create_sync_objects(device: &ash::Device, swapchain_images_len: usize) -> SyncObjects {
        let mut images_in_flight = Vec::with_capacity(swapchain_images_len);
        images_in_flight.resize(swapchain_images_len, None);

        SyncObjects {
            image_availabe_semaphores: [
                unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }
                    .unwrap(),
                unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }
                    .unwrap(),
            ],
            render_finished_semaphores: [
                unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }
                    .unwrap(),
                unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) }
                    .unwrap(),
            ],
            in_flight_fences: [
                unsafe {
                    device.create_fence(
                        &vk::FenceCreateInfo {
                            flags: vk::FenceCreateFlags::SIGNALED,
                            ..Default::default()
                        },
                        None,
                    )
                }
                .unwrap(),
                unsafe {
                    device.create_fence(
                        &vk::FenceCreateInfo {
                            flags: vk::FenceCreateFlags::SIGNALED,
                            ..Default::default()
                        },
                        None,
                    )
                }
                .unwrap(),
            ],
            images_in_flight,
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.device.device.queue_wait_idle(self.queue) }.unwrap();

        self.sync_objects
            .image_availabe_semaphores
            .iter()
            .for_each(|semaphore| unsafe {
                self.device.device.destroy_semaphore(*semaphore, None)
            });
        self.sync_objects
            .render_finished_semaphores
            .iter()
            .for_each(|semaphore| unsafe {
                self.device.device.destroy_semaphore(*semaphore, None)
            });
        self.sync_objects
            .in_flight_fences
            .iter()
            .for_each(|fence| unsafe { self.device.device.destroy_fence(*fence, None) });
        unsafe {
            self.device
                .device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            self.device
                .device
                .destroy_command_pool(self.command_pool, None);
        }
        self.framebuffers.iter().for_each(|framebuffer| unsafe {
            self.device.device.destroy_framebuffer(*framebuffer, None)
        });
        unsafe {
            self.device.device.destroy_pipeline(self.pipeline, None);
            self.device
                .device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .device
                .destroy_render_pass(self.render_pass, None)
        };
        self.swapchain_image_views
            .iter()
            .for_each(|image_view| unsafe {
                self.device.device.destroy_image_view(*image_view, None)
            });
        unsafe {
            self.swapchain.0.destroy_swapchain(self.swapchain.1, None);
            self.device.device.destroy_device(None);
            self.surface
                .surface_loader
                .destroy_surface(self.surface.surface, None);
            self.instance.instance.destroy_instance(None);
        }
    }
}
