use ash::version::EntryV1_0;
use ash::vk;

pub struct Instance {
    pub instance: ash::Instance,
}

impl Instance {
    pub fn new(entry: &super::Entry, window: &winit::window::Window) -> Self {
        Self {
            instance: Self::create_instance(entry, window),
        }
    }

    fn create_instance(entry: &super::Entry, window: &winit::window::Window) -> ash::Instance {
        unsafe {
            entry.entry.create_instance(
                &vk::InstanceCreateInfo {
                    enabled_layer_count: 1,
                    pp_enabled_layer_names: [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as _]
                        .as_ptr(),
                    enabled_extension_count: 2,
                    pp_enabled_extension_names: ash_window::enumerate_required_extensions(window)
                        .unwrap()
                        .iter()
                        .map(|extension| extension.as_ptr())
                        .collect::<Vec<_>>()
                        .as_ptr(),
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap()
    }
}
