use ash::vk;

pub struct SyncObjects {
    pub image_availabe_semaphores: [vk::Semaphore; super::MAX_FRAMES_IN_FLIGHT],
    pub render_finished_semaphores: [vk::Semaphore; super::MAX_FRAMES_IN_FLIGHT],
    pub in_flight_fences: [vk::Fence; super::MAX_FRAMES_IN_FLIGHT],
    pub images_in_flight: Vec<Option<vk::Fence>>,
}

impl SyncObjects {
    pub fn new(device: &super::Device, swapchain: &super::Swapchain) -> Self {
        let mut images_in_flight = Vec::with_capacity(swapchain.images.len());
        images_in_flight.resize(swapchain.images.len(), None);

        Self {
            image_availabe_semaphores: [
                unsafe {
                    device
                        .device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
                .unwrap(),
                unsafe {
                    device
                        .device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
                .unwrap(),
            ],
            render_finished_semaphores: [
                unsafe {
                    device
                        .device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
                .unwrap(),
                unsafe {
                    device
                        .device
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
                .unwrap(),
            ],
            in_flight_fences: [
                unsafe {
                    device.device.create_fence(
                        &vk::FenceCreateInfo {
                            flags: vk::FenceCreateFlags::SIGNALED,
                            ..Default::default()
                        },
                        None,
                    )
                }
                .unwrap(),
                unsafe {
                    device.device.create_fence(
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

    pub fn destroy(&self, device: &super::Device) {
        self.image_availabe_semaphores
            .iter()
            .for_each(|semaphore| unsafe { device.device.destroy_semaphore(*semaphore, None) });
        self.render_finished_semaphores
            .iter()
            .for_each(|semaphore| unsafe { device.device.destroy_semaphore(*semaphore, None) });
        self.in_flight_fences
            .iter()
            .for_each(|fence| unsafe { device.device.destroy_fence(*fence, None) });
    }
}
