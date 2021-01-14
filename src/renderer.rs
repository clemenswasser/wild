mod buffer;
mod command_buffers;
mod command_pool;
mod debug_utils;
mod device;
mod entry;
mod framebuffers;
mod index_buffer;
mod instance;
mod pipeline;
mod render_pass;
mod surface;
mod swapchain;
mod sync_objects;
mod uniform_object;
mod vertex;
mod vertex_buffer;

use buffer::Buffer;
use command_buffers::CommandBuffers;
use command_pool::CommandPool;
#[cfg(debug_assertions)]
use debug_utils::DebugUtils;
use device::Device;
use entry::Entry;
use framebuffers::Framebuffers;
use index_buffer::IndexBuffer;
use instance::Instance;
use pipeline::Pipeline;
use render_pass::RenderPass;
use surface::Surface;
use swapchain::Swapchain;
use sync_objects::SyncObjects;
use uniform_object::UniformObject;
use vertex::Vertex;
use vertex_buffer::VertexBuffer;

use std::io::Write;

use ash::{version::DeviceV1_0, vk};

const UNIFORM_OBJECT_SIZE: usize = std::mem::size_of::<UniformObject>();
const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    _entry: Entry,
    instance: Instance,
    #[cfg(debug_assertions)]
    debug_utils: DebugUtils,
    surface: Surface,
    device: Device,
    swapchain: Swapchain,
    render_pass: RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline: Pipeline,
    framebuffers: Framebuffers,
    command_pool: CommandPool,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    uniform_buffers: Vec<Buffer>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    command_buffers: CommandBuffers,
    sync_objects: SyncObjects,
    current_frame: usize,
    resized: bool,
    time: std::time::SystemTime,
    frames: u32,
    timer: std::time::SystemTime,
    rotation: f32,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Self {
        let vertices = [
            Vertex::new(cgmath::vec2(-0.5, -0.5), cgmath::vec3(1.0, 0.0, 0.0)),
            Vertex::new(cgmath::vec2(0.5, -0.5), cgmath::vec3(0.0, 1.0, 0.0)),
            Vertex::new(cgmath::vec2(0.5, 0.5), cgmath::vec3(0.0, 0.0, 1.0)),
            Vertex::new(cgmath::vec2(-0.5, 0.5), cgmath::vec3(1.0, 1.0, 1.0)),
        ];
        let indices = [0, 1, 2, 2, 3, 0];
        let entry = Entry::new();
        let instance = Instance::new(&entry, window);
        #[cfg(debug_assertions)]
        let debug_utils = DebugUtils::new(&entry, &instance);
        let mut surface = Surface::new(window, &entry, &instance);
        let device = Device::new(&instance, &mut surface);
        let swapchain = Swapchain::new(&instance, &surface, &device);
        let render_pass = RenderPass::new(&surface, &device);
        let descriptor_set_layout = Self::create_descriptor_set_layout(&device);
        let pipeline = Pipeline::new(&surface, &device, &render_pass, &descriptor_set_layout);
        let framebuffers = Framebuffers::new(&surface, &device, &swapchain, &render_pass);
        let command_pool = CommandPool::new(&device);
        let vertex_buffer = VertexBuffer::new(&instance, &device, &command_pool, vertices.into());
        let index_buffer = IndexBuffer::new(&instance, &device, &command_pool, indices.into());
        let uniform_buffers = Self::create_uniform_buffers(&instance, &device, &swapchain);
        let descriptor_pool = Self::create_descriptor_pool(&device, &swapchain);
        let descriptor_sets = Self::create_descriptor_sets(
            &device,
            &swapchain,
            &descriptor_set_layout,
            &uniform_buffers,
            &descriptor_pool,
        );
        let command_buffers = CommandBuffers::new(
            &surface,
            &device,
            &render_pass,
            &pipeline,
            &framebuffers,
            &command_pool,
            &vertex_buffer,
            &index_buffer,
            &descriptor_sets,
        );
        let sync_objects = SyncObjects::new(&device, &swapchain);
        Self {
            _entry: entry,
            instance,
            #[cfg(debug_assertions)]
            debug_utils,
            surface,
            device,
            swapchain,
            render_pass,
            descriptor_set_layout,
            pipeline,
            framebuffers,
            command_pool,
            vertex_buffer,
            index_buffer,
            uniform_buffers,
            descriptor_pool,
            descriptor_sets,
            command_buffers,
            sync_objects,
            current_frame: 0,
            resized: false,
            time: std::time::SystemTime::now(),
            frames: 0,
            timer: std::time::SystemTime::now(),
            rotation: 0f32,
        }
    }

    pub fn render(&mut self) {
        if self.time.elapsed().unwrap().as_millis() > 1000 {
            print!("\r{} FPS", self.frames);
            let _ = std::io::stdout().flush();
            self.time = std::time::SystemTime::now();
            self.frames = 0;
        }
        self.frames += 1;

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

        let image_index = match unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.swapchain,
                std::u64::MAX,
                *self
                    .sync_objects
                    .image_availabe_semaphores
                    .get(self.current_frame)
                    .unwrap(),
                vk::Fence::null(),
            )
        } {
            Ok((image_index, _)) => image_index,
            Err(result) => {
                if result == vk::Result::ERROR_OUT_OF_DATE_KHR {
                    self.recreate_swapchain();
                    self.render();
                    return;
                } else {
                    panic!("Failed to acuire next image!");
                }
            }
        };

        self.update_uniform_buffer(image_index as _);

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
                self.device.queue,
                &[vk::SubmitInfo {
                    wait_semaphore_count: 1,
                    p_wait_semaphores: self
                        .sync_objects
                        .image_availabe_semaphores
                        .get(self.current_frame)
                        .unwrap(),
                    p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    command_buffer_count: 1,
                    p_command_buffers: self
                        .command_buffers
                        .command_buffers
                        .get(image_index as usize)
                        .unwrap(),
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

        match unsafe {
            self.swapchain.loader.queue_present(
                self.device.queue,
                &vk::PresentInfoKHR {
                    wait_semaphore_count: 1,
                    p_wait_semaphores: self
                        .sync_objects
                        .render_finished_semaphores
                        .get(self.current_frame)
                        .unwrap(),
                    swapchain_count: 1,
                    p_swapchains: &self.swapchain.swapchain,
                    p_image_indices: &image_index,
                    ..Default::default()
                },
            )
        } {
            Ok(suboptimal) => {
                if suboptimal {
                    self.resized = false;
                    self.recreate_swapchain();
                }
            }
            Err(result) => {
                if result == vk::Result::ERROR_OUT_OF_DATE_KHR {
                    self.resized = false;
                    self.recreate_swapchain();
                    self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
                    self.render();
                    return;
                } else {
                    panic!("Failed to present!");
                }
            }
        };

        if self.resized {
            self.resized = false;
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn update_uniform_buffer(&mut self, current_image: usize) {
        let current_time = std::time::SystemTime::now();
        self.rotation += current_time
            .duration_since(self.timer)
            .unwrap()
            .as_secs_f32() as f32
            * 90f32;
        let uniform_object = UniformObject {
            model: cgmath::Matrix4::from_angle_z(cgmath::Deg(self.rotation)),
            view: cgmath::Matrix4::look_at_rh(
                cgmath::Point3::new(2f32, 2f32, 2f32),
                cgmath::Point3::new(0f32, 0f32, 0f32),
                cgmath::Vector3::new(0f32, 0f32, 1.0),
            ),
            projection: cgmath::perspective(
                cgmath::Deg(45f32),
                self.surface.capabilities.unwrap().current_extent.width as f32
                    / self.surface.capabilities.unwrap().current_extent.height as f32,
                0.1f32,
                10f32,
            ),
        };

        self.timer = current_time;

        self.uniform_buffers
            .get(current_image)
            .unwrap()
            .write(&self.device, uniform_object);
    }

    fn recreate_swapchain(&mut self) {
        unsafe { self.device.device.device_wait_idle() }.unwrap();

        self.framebuffers.destroy(&self.device);
        self.command_buffers.free(&self.device, &self.command_pool);
        self.pipeline.destroy(&self.device);
        self.render_pass.destroy(&self.device);
        self.swapchain.destroy(&self.device);
        self.uniform_buffers
            .iter()
            .for_each(|uniform_buffer| uniform_buffer.destory(&self.device));
        unsafe {
            self.device
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None)
        };

        self.surface
            .update_format_and_capabilities(&self.device.physical_device);
        self.swapchain = Swapchain::new(&self.instance, &self.surface, &self.device);
        self.render_pass = RenderPass::new(&self.surface, &self.device);
        self.pipeline = Pipeline::new(
            &self.surface,
            &self.device,
            &self.render_pass,
            &self.descriptor_set_layout,
        );
        self.framebuffers = Framebuffers::new(
            &self.surface,
            &self.device,
            &self.swapchain,
            &self.render_pass,
        );
        self.uniform_buffers =
            Self::create_uniform_buffers(&self.instance, &self.device, &self.swapchain);
        self.descriptor_pool = Self::create_descriptor_pool(&self.device, &self.swapchain);
        self.descriptor_sets = Self::create_descriptor_sets(
            &self.device,
            &self.swapchain,
            &self.descriptor_set_layout,
            &self.uniform_buffers,
            &self.descriptor_pool,
        );
        self.command_buffers = CommandBuffers::new(
            &self.surface,
            &self.device,
            &self.render_pass,
            &self.pipeline,
            &self.framebuffers,
            &self.command_pool,
            &self.vertex_buffer,
            &self.index_buffer,
            &self.descriptor_sets,
        );
    }

    fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
        unsafe {
            device.device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo {
                    binding_count: 1,
                    p_bindings: &vk::DescriptorSetLayoutBinding {
                        binding: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                        stage_flags: vk::ShaderStageFlags::VERTEX,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap()
    }

    fn create_uniform_buffers(
        instance: &Instance,
        device: &Device,
        swapchain: &Swapchain,
    ) -> Vec<Buffer> {
        (0..swapchain.images.len())
            .map(|_| {
                Buffer::new(
                    &instance,
                    &device,
                    UNIFORM_OBJECT_SIZE,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
            })
            .collect::<Vec<_>>()
    }

    fn create_descriptor_pool(device: &Device, swapchain: &Swapchain) -> vk::DescriptorPool {
        unsafe {
            device.device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo {
                    pool_size_count: 1,
                    p_pool_sizes: &vk::DescriptorPoolSize {
                        ty: vk::DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: swapchain.images.len() as _,
                    },
                    max_sets: swapchain.images.len() as _,
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap()
    }

    fn create_descriptor_sets(
        device: &Device,
        swapchain: &Swapchain,
        descriptor_set_layout: &vk::DescriptorSetLayout,
        uniform_buffers: &[Buffer],
        descriptor_pool: &vk::DescriptorPool,
    ) -> Vec<vk::DescriptorSet> {
        let descriptor_sets = unsafe {
            device
                .device
                .allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo {
                    descriptor_pool: *descriptor_pool,
                    descriptor_set_count: swapchain.images.len() as _,
                    p_set_layouts: (0..swapchain.images.len())
                        .map(|_| *descriptor_set_layout)
                        .collect::<Vec<_>>()
                        .as_ptr(),
                    ..Default::default()
                })
        }
        .unwrap();
        descriptor_sets
            .iter()
            .enumerate()
            .for_each(|(i, descriptor_set)| unsafe {
                device.device.update_descriptor_sets(
                    &[vk::WriteDescriptorSet {
                        dst_set: *descriptor_set,
                        dst_binding: 0,
                        dst_array_element: 0,
                        descriptor_count: 1,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        p_buffer_info: &vk::DescriptorBufferInfo {
                            buffer: uniform_buffers.get(i).unwrap().buffer,
                            offset: 0,
                            range: UNIFORM_OBJECT_SIZE as _,
                        },
                        ..Default::default()
                    }],
                    &[],
                )
            });
        descriptor_sets
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.device.device.queue_wait_idle(self.device.queue) }.unwrap();

        unsafe {
            self.device
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None)
        };
        self.index_buffer.destory(&self.device);
        self.vertex_buffer.destory(&self.device);
        self.sync_objects.destroy(&self.device);
        self.command_buffers.free(&self.device, &self.command_pool);
        self.command_pool.destroy(&self.device);
        self.framebuffers.destroy(&self.device);
        self.pipeline.destroy(&self.device);
        unsafe {
            self.device
                .device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None)
        };
        self.render_pass.destroy(&self.device);
        self.swapchain.destroy(&self.device);
        self.uniform_buffers
            .iter()
            .for_each(|uniform_buffer| uniform_buffer.destory(&self.device));
        self.device.destroy();
        self.surface.destroy();
        #[cfg(debug_assertions)]
        self.debug_utils.destroy();
        self.instance.destroy();
    }
}
