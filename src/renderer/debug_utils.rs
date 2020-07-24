#![cfg(debug_assertions)]

use super::{Entry, Instance};
use ash::vk;

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

pub struct DebugUtils {
    loader: ash::extensions::ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl DebugUtils {
    pub fn new(entry: &Entry, instance: &Instance) -> Self {
        let loader = ash::extensions::ext::DebugUtils::new(&entry.entry, &instance.instance);
        #[cfg(debug_assertions)]
        let messenger = unsafe {
            loader.create_debug_utils_messenger(
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
        Self { loader, messenger }
    }

    pub fn destroy(&self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None)
        };
    }
}
