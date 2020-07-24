mod command_buffers;
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
mod vertex;
mod vertex_buffer;

use command_buffers::CommandBuffers;
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
use vertex::Vertex;
use vertex_buffer::VertexBuffer;

use ash::{version::DeviceV1_0, vk};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[cfg(debug_assertions)]
unsafe extern "system" fn debug_utils_messenger_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    println!(
        "{}",
        std::ffi::CStr::from_ptr(callback_data.p_message)
            .to_str()
            .unwrap()
    );
    vk::FALSE
}

pub struct Renderer {
    pub entry: Entry,
    pub instance: Instance,
    #[cfg(debug_assertions)]
    pub debug_utils: (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT),
    pub surface: Surface,
    pub device: Device,
    pub swapchain: Swapchain,
    pub render_pass: RenderPass,
    pub pipeline: Pipeline,
    pub framebuffers: Framebuffers,
    pub command_pool: CommandPool,
    pub command_buffers: CommandBuffers,
    pub sync_objects: SyncObjects,
    pub current_frame: usize,
    pub resized: bool,
    pub time: std::time::SystemTime,
    pub frames: u32,
    vertex_buffer: VertexBuffer,
}

impl Renderer {
    pub fn new(window: &winit::window::Window) -> Self {
        let vertices = [
            Vertex::new(cgmath::vec2(0.0, -0.5), cgmath::vec3(1.0, 0.0, 0.0)),
            Vertex::new(cgmath::vec2(0.5, 0.5), cgmath::vec3(0.0, 1.0, 0.0)),
            Vertex::new(cgmath::vec2(-0.5, 0.5), cgmath::vec3(0.0, 0.0, 1.0)),
        ];
        let entry = Entry::new();
        let instance = Instance::new(&entry, window);
        #[cfg(debug_assertions)]
        let debug_utils_loader =
            ash::extensions::ext::DebugUtils::new(&entry.entry, &instance.instance);
        #[cfg(debug_assertions)]
        let debug_utils_messenger = unsafe {
            debug_utils_loader.create_debug_utils_messenger(
                &vk::DebugUtilsMessengerCreateInfoEXT {
                    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                    message_type: vk::DebugUtilsMessageTypeFlagsEXT::all(),
                    pfn_user_callback: Some(debug_utils_messenger_callback),
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        let mut surface = Surface::new(window, &entry, &instance);
        let device = Device::new(&instance, &mut surface);
        let swapchain = Swapchain::new(&instance, &surface, &device);
        let render_pass = RenderPass::new(&surface, &device);
        let pipeline = Pipeline::new(&surface, &device, &render_pass);
        let framebuffers = Framebuffers::new(&surface, &device, &swapchain, &render_pass);
        let command_pool = CommandPool::new(&device);
        let vertex_buffer = VertexBuffer::new(&instance, &device, &command_pool, vertices.into());
        let command_buffers = CommandBuffers::new(
            &surface,
            &device,
            &render_pass,
            &pipeline,
            &framebuffers,
            &command_pool,
            &vertex_buffer,
        );
        let sync_objects = SyncObjects::new(&device, &swapchain);
        Self {
            entry,
            instance,
            #[cfg(debug_assertions)]
            debug_utils: (debug_utils_loader, debug_utils_messenger),
            surface,
            device,
            swapchain,
            render_pass,
            pipeline,
            framebuffers,
            command_pool,
            command_buffers,
            sync_objects,
            current_frame: 0,
            resized: false,
            time: std::time::SystemTime::now(),
            frames: 0,
            vertex_buffer,
        }
    }

    pub fn render(&mut self) {
        self.frames += 1;
        if self.time.elapsed().unwrap().as_millis() > 1000 {
            println!("{} FPS", self.frames);
            self.time = std::time::SystemTime::now();
            self.frames = 0;
        }

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

    fn recreate_swapchain(&mut self) {
        unsafe { self.device.device.device_wait_idle() }.unwrap();

        self.framebuffers.destroy(&self.device);
        self.command_buffers.free(&self.device, &self.command_pool);
        self.pipeline.destroy(&self.device);
        self.render_pass.destroy(&self.device);
        self.swapchain.destroy(&self.device);

        self.surface
            .update_format_and_capabilities(&self.device.physical_device);
        self.swapchain = Swapchain::new(&self.instance, &self.surface, &self.device);
        self.render_pass = RenderPass::new(&self.surface, &self.device);
        self.pipeline = Pipeline::new(&self.surface, &self.device, &self.render_pass);
        self.framebuffers = Framebuffers::new(
            &self.surface,
            &self.device,
            &self.swapchain,
            &self.render_pass,
        );
        self.command_buffers = CommandBuffers::new(
            &self.surface,
            &self.device,
            &self.render_pass,
            &self.pipeline,
            &self.framebuffers,
            &self.command_pool,
            &self.vertex_buffer,
        );
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe { self.device.device.queue_wait_idle(self.device.queue) }.unwrap();
        self.vertex_buffer.destory(&self.device);
        self.sync_objects.destroy(&self.device);
        self.command_buffers.free(&self.device, &self.command_pool);
        self.command_pool.destroy(&self.device);
        self.framebuffers.destroy(&self.device);
        self.pipeline.destroy(&self.device);
        self.render_pass.destroy(&self.device);
        self.swapchain.destroy(&self.device);
        self.device.destroy();
        self.surface.destroy();
        #[cfg(debug_assertions)]
        unsafe {
            self.debug_utils
                .0
                .destroy_debug_utils_messenger(self.debug_utils.1, None)
        };
        self.instance.destroy();
    }
}
