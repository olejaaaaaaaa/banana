use ash::vk;
use log::debug;
use std::ffi::CStr;

use crate::VulkanResult;

const ENGINE_VERSION: u32 = 0;
const ENGINE_NAME: &'static CStr = c"Ferrum";

pub struct App {
    pub(crate) create_info: vk::ApplicationInfo<'static>,
    pub(crate) entry: ash::Entry,
}

pub struct AppBuilder {
    api_version: u32,
    app_name: &'static CStr,
    app_version: u32,
}

impl AppBuilder {
    
    pub fn default() -> Self {
        AppBuilder { app_name: c"App", app_version: 0, api_version: vk::API_VERSION_1_0 }
    }

    pub fn with_api_version(mut self, version: u32) -> Self {
        self.api_version = version;
        self
    }

    pub fn with_app_name(mut self, name: &'static CStr) -> Self {
        self.app_name = name;
        self
    }

    pub fn build(self) -> VulkanResult<App> {

        let entry = unsafe { ash::Entry::load().expect("Error load entry point") };
        let max_api_versions = unsafe {
            entry
                .try_enumerate_instance_version()
                .expect("Error enumerate api veriosn")
                .unwrap_or(vk::API_VERSION_1_0)
        };

        if self.api_version > max_api_versions {
            panic!("R");
        }

        let create_info = vk::ApplicationInfo::default()
            .api_version(self.api_version)
            .application_name(self.app_name)
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION);

        debug!("App: {:?}", create_info);

        Ok(App { create_info, entry })
    }
}


