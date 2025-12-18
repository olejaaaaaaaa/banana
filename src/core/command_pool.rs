
use ash::vk;
use crate::{VulkanError, VulkanResult, core::device::Device};

pub struct CommandPool {
    raw: vk::CommandPool
}

impl CommandPool {
    pub fn create_command_buffers(&self, device: &Device, count: u32) -> VulkanResult<Vec<vk::CommandBuffer>> {

        let create_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.raw)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        let buffers = unsafe {
            device.allocate_command_buffers(&create_info)
                .map_err(|e| VulkanError::Unknown(e))
        }?;

        Ok(buffers)
    }
}

pub struct CommandPoolBuilder<'a> {
    device: &'a Device,
    create_info: vk::CommandPoolCreateInfo<'static>
}

impl<'a> CommandPoolBuilder<'a> {

    pub fn reset(device: &'a Device) -> Self {
        Self { 
            device,
            create_info: vk::CommandPoolCreateInfo::default().flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER) 
        }
    }

    pub fn build(self) -> VulkanResult<CommandPool> {

        puffin::profile_scope!("vkCommandBuffers");

        let pool = unsafe { self.device.create_command_pool(&self.create_info, None).map_err(|e| {
            VulkanError::Unknown(e)
        })}?;

        Ok(CommandPool { raw: pool })
    }
}
