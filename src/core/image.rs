
use crate::{VulkanResult, core::device::Device};
use ash::vk;
use vk_mem::Alloc;

pub struct Image {
    pub(crate) raw: vk::Image,
    allocation: vk_mem::Allocation
}

impl Image {
    pub fn destory(&mut self, device: &Device) {
        unsafe {
            device.allocator.destroy_image(self.raw, &mut self.allocation);
        }
    }
}

pub struct ImageBuilder<'a> {
    pub device: &'a Device,
    pub create_info: vk::ImageCreateInfo<'static>,
    pub alloc_info: vk_mem::AllocationCreateInfo,
}

impl<'a> ImageBuilder<'a> {
    pub fn depth(device: &'a Device, format: vk::Format, extent: vk::Extent2D) -> Self {
        Self { 
            device,
            create_info: vk::ImageCreateInfo::default()
                .mip_levels(1)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .extent(extent.into())
                .format(format)
                .tiling(vk::ImageTiling::OPTIMAL)
                .samples(vk::SampleCountFlags::TYPE_1)
                .array_layers(1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .image_type(vk::ImageType::TYPE_2D), 
            alloc_info: vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                ..Default::default()
            } 
        }
    }

    pub fn build(self) -> VulkanResult<Image> {
        let (image, allocation) = unsafe { 
            self.device.allocator.create_image(&self.create_info, &self.alloc_info).unwrap() 
        };
        Ok(Image { raw: image, allocation })
    }
}

