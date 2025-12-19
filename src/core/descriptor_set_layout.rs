

use ash::vk;

use crate::{Device, VulkanError, VulkanResult};

pub struct DescriptorSetLayout {
    pub raw: vk::DescriptorSetLayout
}

pub struct DescriptorSetLayoutBuilder<'a> {
    device: &'a Device,
    bindings: Vec<vk::DescriptorSetLayoutBinding<'static>>
}

impl<'a> DescriptorSetLayoutBuilder<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self { 
            bindings: vec![],
            device,
        }
    }

    pub fn bindings(mut self, bindings: Vec<vk::DescriptorSetLayoutBinding<'static>>) -> Self {
        self.bindings = bindings;
        self
    }

    pub fn build(self) -> VulkanResult<DescriptorSetLayout> {

        let create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&self.bindings);

        let layout = unsafe {
            self.device.create_descriptor_set_layout(&create_info, None)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        Ok(DescriptorSetLayout { raw: layout })
    }
}