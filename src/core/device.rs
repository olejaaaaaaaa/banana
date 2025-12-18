use std::{ffi::CStr, mem::ManuallyDrop};

use ash::vk;
use vk_mem::Allocator;

use crate::{Instance, VulkanResult};

pub struct Device {
    pub(crate) allocator: ManuallyDrop<Allocator>,
    pub(crate) queue_family_props: Vec<vk::QueueFamilyProperties>,
    pub(crate) raw: ash::Device,
}

impl Device {
    pub fn destroy(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
            self.raw.destroy_device(None);
        }
    }
}

impl std::ops::Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &ash::Device {
        &self.raw
    }
}

pub struct DeviceBuilder<'a> {
    phys_dev: &'a vk::PhysicalDevice,
    instance: &'a Instance,
    extenions: Vec<&'static CStr>
}

impl<'a> DeviceBuilder<'a> {

    pub fn default(instance: &'a Instance, phys_dev: &'a vk::PhysicalDevice) -> Self {
        DeviceBuilder { 
            instance, 
            phys_dev, 
            extenions: vec![c"VK_KHR_swapchain"] 
        }
    }

    pub fn build(self) -> VulkanResult<Device> {

        let queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(0)
            .queue_priorities(&[1.0]);

        let p_extenions = self.extenions.iter().map(|p| p.as_ptr() as *const i8).collect::<Vec<_>>();

        let binding = [queue_create_info];
        let create_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&p_extenions)
            .queue_create_infos(&binding);

        let device = unsafe {
            self.instance.raw
                .create_device(*self.phys_dev, &create_info, None)
                .unwrap()
        };

        let queue_prop = unsafe { self.instance.raw.get_physical_device_queue_family_properties(*self.phys_dev) };
        let create_info = vk_mem::AllocatorCreateInfo::new(&self.instance.raw, &device, *self.phys_dev);
        let allocator = unsafe { vk_mem::Allocator::new(create_info).unwrap() };

        Ok(Device {
            raw: device,
            allocator: ManuallyDrop::new(allocator),
            queue_family_props: queue_prop
        })
    }
}

