use ash::version::InstanceV1_0;
use ash::vk;

pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
}

impl Device {
    pub fn new(instance: &super::Instance, surface: &mut super::Surface) -> Self {
        let physical_devices = unsafe { instance.instance.enumerate_physical_devices() }.unwrap();
        let physical_device = physical_devices
            .iter()
            .find(|physical_device| surface.is_supported(physical_device))
            .unwrap();

        let device = unsafe {
            instance.instance.create_device(
                *physical_device,
                &vk::DeviceCreateInfo {
                    queue_create_info_count: 1,
                    p_queue_create_infos: &vk::DeviceQueueCreateInfo {
                        queue_family_index: 0,
                        queue_count: 1,
                        p_queue_priorities: &1.0,
                        ..Default::default()
                    },
                    enabled_extension_count: 1,
                    pp_enabled_extension_names: [ash::extensions::khr::Swapchain::name().as_ptr()]
                        .as_ptr(),
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap();

        Self {
            physical_device: *physical_device,
            device,
        }
    }
}
