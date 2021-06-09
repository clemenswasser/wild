use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

pub struct Device {
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub queue: vk::Queue,
}

impl Device {
    pub fn new(instance: &super::Instance, surface: &mut super::Surface) -> Self {
        let physical_devices = unsafe { instance.instance.enumerate_physical_devices() }.unwrap();
        let physical_device = physical_devices
            .iter()
            .find(|physical_device| surface.is_supported(physical_device))
            .unwrap();

        #[cfg(debug_assertions)]
        {
            let physical_device_properties = unsafe {
                instance
                    .instance
                    .get_physical_device_properties(*physical_device)
            };
            println!(
                "{} (api_version: {}.{}.{})",
                unsafe {
                    std::ffi::CStr::from_ptr(physical_device_properties.device_name.as_ptr())
                }
                .to_str()
                .unwrap(),
                vk::version_major(physical_device_properties.api_version),
                vk::version_minor(physical_device_properties.api_version),
                vk::version_patch(physical_device_properties.api_version)
            );
        }

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
        let queue = unsafe { device.get_device_queue(0, 0) };

        Self {
            physical_device: *physical_device,
            device,
            queue,
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) };
    }
}
