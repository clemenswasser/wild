mod command_pool;
mod device;
mod entry;
mod framebuffers;
mod instance;
mod pipeline;
mod render_pass;
mod surface;
mod swapchain;
mod sync_objects;

use command_pool::CommandPool;
use device::Device;
use entry::Entry;
use framebuffers::Framebuffers;
use instance::Instance;
use pipeline::Pipeline;
use render_pass::RenderPass;
use surface::Surface;
use swapchain::Swapchain;
use sync_objects::SyncObjects;

use ash::{version::DeviceV1_0, vk};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct Renderer {
    pub entry: Entry,
    pub instance: Instance,
    pub surface: Surface,
    pub device: Device,
    pub swapchain: Swapchain,
    pub render_pass: RenderPass,
    pub pipeline: Pipeline,
    pub framebuffers: Framebuffers,
    pub command_pool: CommandPool,
    pub sync_objects: SyncObjects,
    pub current_frame: usize,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = Entry::new();
        let instance = Instance::new(&entry, window);
        let mut surface = Surface::new(window, &entry, &instance);
        let device = Device::new(&instance, &mut surface);
        let swapchain = Swapchain::new(&instance, &surface, &device);
        let render_pass = RenderPass::new(&surface, &device);
        let pipeline = Pipeline::new(&surface, &device, &render_pass);
        let framebuffers = Framebuffers::new(&surface, &device, &swapchain, &render_pass);
        let command_pool =
            CommandPool::new(&surface, &device, &render_pass, &pipeline, &framebuffers);
        let sync_objects = SyncObjects::new(&device, &swapchain);
        Self {
            entry,
            instance,
            surface,
            device,
            swapchain,
            render_pass,
            pipeline,
            framebuffers,
            command_pool,
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
                        .command_pool
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

        unsafe {
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
        }
        .unwrap();
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.device.device.queue_wait_idle(self.device.queue) }.unwrap();
        self.sync_objects.destroy(&self.device);
        self.command_pool.destroy(&self.device);
        self.framebuffers.destroy(&self.device);
        self.pipeline.destroy(&self.device);
        self.render_pass.destroy(&self.device);
        self.swapchain.destroy(&self.device);
        self.device.destroy();
        self.surface.destroy();
        self.instance.destroy();
    }
}
