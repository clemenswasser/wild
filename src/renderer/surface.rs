use ash::vk;

pub struct Surface {
    pub loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
    pub capabilities: Option<vk::SurfaceCapabilitiesKHR>,
    pub format: Option<vk::SurfaceFormatKHR>,
}

impl Surface {
    pub fn new(
        window: &winit::window::Window,
        entry: &super::Entry,
        instance: &super::Instance,
    ) -> Self {
        let loader = ash::extensions::khr::Surface::new(&entry.entry, &instance.instance);
        let surface =
            unsafe { ash_window::create_surface(&entry.entry, &instance.instance, window, None) }
                .unwrap();
        Self {
            loader,
            surface,
            capabilities: None,
            format: None,
        }
    }

    pub fn is_supported(&mut self, physical_device: &vk::PhysicalDevice) -> bool {
        let res = unsafe {
            self.loader
                .get_physical_device_surface_support(*physical_device, 0, self.surface)
        }
        .unwrap();
        if res {
            self.update_format_and_capabilities(physical_device);
        }
        res
    }

    pub fn update_format_and_capabilities(&mut self, physical_device: &vk::PhysicalDevice) {
        self.capabilities = Some(
            unsafe {
                self.loader
                    .get_physical_device_surface_capabilities(*physical_device, self.surface)
            }
            .unwrap(),
        );
        self.format = Some(self.get_surface_format(physical_device));
    }

    fn get_surface_format(&self, physical_device: &vk::PhysicalDevice) -> vk::SurfaceFormatKHR {
        let mut surface_formats = unsafe {
            self.loader
                .get_physical_device_surface_formats(*physical_device, self.surface)
        }
        .unwrap();

        let surface_format: vk::SurfaceFormatKHR = match surface_formats
            .iter()
            .filter(|surface_format| {
                surface_format
                    .color_space
                    .eq(&vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    && surface_format.format.eq(&vk::Format::B8G8R8A8_SRGB)
            })
            .collect::<Vec<_>>()
            .get(0)
        {
            Some(surface_format) => **surface_format,
            None => {
                let surface_format = surface_formats.remove(0);
                println!("Using a fallback surface format: {:?}", &surface_format);
                surface_format
            }
        };

        surface_format
    }

    pub fn destroy(&self) {
        unsafe { self.loader.destroy_surface(self.surface, None) };
    }
}
