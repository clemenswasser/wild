use ash::version::DeviceV1_0;
use ash::vk;

pub struct Pipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn new(
        surface: &super::Surface,
        device: &super::Device,
        render_pass: &super::RenderPass,
        descriptor_set_layout: &vk::DescriptorSetLayout,
    ) -> Self {
        let vertex_shader = include_bytes!("../../shaders/vert.spv");
        let fragment_shader = include_bytes!("../../shaders/frag.spv");

        let vertex_shader_module = unsafe {
            device.device.create_shader_module(
                #[allow(clippy::cast_ptr_alignment)]
                &vk::ShaderModuleCreateInfo {
                    code_size: vertex_shader.len(),
                    p_code: vertex_shader.as_ptr().cast(),
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        let fragment_shader_module = unsafe {
            device.device.create_shader_module(
                #[allow(clippy::cast_ptr_alignment)]
                &vk::ShaderModuleCreateInfo {
                    code_size: fragment_shader.len(),
                    p_code: fragment_shader.as_ptr().cast(),
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        let layout = unsafe {
            device.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo {
                    set_layout_count: 1,
                    p_set_layouts: descriptor_set_layout,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        let pipeline = unsafe {
            device.device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[vk::GraphicsPipelineCreateInfo {
                    stage_count: 2,
                    p_stages: [
                        vk::PipelineShaderStageCreateInfo {
                            stage: vk::ShaderStageFlags::VERTEX,
                            module: vertex_shader_module,
                            p_name: b"main\0".as_ptr().cast(),
                            ..Default::default()
                        },
                        vk::PipelineShaderStageCreateInfo {
                            stage: vk::ShaderStageFlags::FRAGMENT,
                            module: fragment_shader_module,
                            p_name: b"main\0".as_ptr().cast(),
                            ..Default::default()
                        },
                    ]
                    .as_ptr(),
                    p_vertex_input_state: &vk::PipelineVertexInputStateCreateInfo {
                        vertex_binding_description_count: 1,
                        p_vertex_binding_descriptions: &super::Vertex::binding_description(),
                        vertex_attribute_description_count: 2,
                        p_vertex_attribute_descriptions: super::Vertex::attribute_descriptions()
                            .as_ptr(),
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
                            width: surface.capabilities.unwrap().current_extent.width as _,
                            height: surface.capabilities.unwrap().current_extent.height as _,
                            min_depth: 0.0,
                            max_depth: 1.0,
                        },
                        scissor_count: 1,
                        p_scissors: &vk::Rect2D {
                            offset: vk::Offset2D { x: 0, y: 0 },
                            extent: surface.capabilities.unwrap().current_extent,
                        },
                        ..Default::default()
                    },
                    p_rasterization_state: &vk::PipelineRasterizationStateCreateInfo {
                        depth_clamp_enable: vk::FALSE,
                        rasterizer_discard_enable: vk::FALSE,
                        polygon_mode: vk::PolygonMode::FILL,
                        line_width: 1.0,
                        cull_mode: vk::CullModeFlags::FRONT,
                        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
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
                            color_write_mask: vk::ColorComponentFlags::all(),
                            blend_enable: vk::FALSE,
                            ..Default::default()
                        },
                        blend_constants: [0.0, 0.0, 0.0, 0.0],
                        ..Default::default()
                    },
                    layout,
                    render_pass: render_pass.render_pass,
                    subpass: 0,
                    ..Default::default()
                }],
                None,
            )
        }
        .unwrap()
        .remove(0);

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

        Self { layout, pipeline }
    }

    pub fn destroy(&self, device: &super::Device) {
        unsafe {
            device.device.destroy_pipeline(self.pipeline, None);
            device.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
