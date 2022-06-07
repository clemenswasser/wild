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
            .to_vec();
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
                    pp_enabled_layer_names: [b"VK_LAYER_KHRONOS_validation\0".as_ptr().cast()]
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
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}
