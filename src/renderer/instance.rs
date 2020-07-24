use ash::version::{EntryV1_0, InstanceV1_0};
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
        let extensions = ash_window::enumerate_required_extensions(window)
            .unwrap()
            .iter()
            .map(|extension| extension.as_ptr())
            .collect::<Vec<_>>();
        #[cfg(debug_assertions)]
        let mut extensions = extensions;
        #[cfg(debug_assertions)]
        extensions.push(ash::extensions::ext::DebugUtils::name().as_ptr());

        unsafe {
            entry.entry.create_instance(
                &vk::InstanceCreateInfo {
                    #[cfg(debug_assertions)]
                    enabled_layer_count: 1,
                    #[cfg(debug_assertions)]
                    pp_enabled_layer_names: [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as _]
                        .as_ptr(),
                    enabled_extension_count: extensions.len() as _,
                    pp_enabled_extension_names: extensions.as_ptr(),
                    ..Default::default()
                },
                None,
            )
        }
        .unwrap()
    }

    pub fn destroy(&self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}