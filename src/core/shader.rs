use std::{error::Error, io::Read, path::Path};
use crate::{VulkanError, VulkanResult, core::device::Device};
use ash::vk;

pub struct ShaderModule {
    pub(crate) raw: vk::ShaderModule
}

pub struct ShaderBuilder<'a, S: AsRef<Path>> {
    device: &'a Device,
    path: S
}

impl<'a, S: AsRef<Path>> ShaderBuilder<'a, S> {
    pub fn from_file(device: &'a Device, path: S) -> VulkanResult<ShaderModule> {
        
        puffin::profile_scope!("vkShaderModule");

        let code = load_spv(path.as_ref());
        let create_info = vk::ShaderModuleCreateInfo::default()
            .code(&code);

        let shader = unsafe { 
            device.create_shader_module(&create_info, None).map_err(|e| {
                VulkanError::Unknown(e)
            })?
        };

        Ok(ShaderModule { raw: shader })
    }
}

pub(crate) fn read_shader_from_bytes(bytes: &[u8]) -> Result<Vec<u32>, Box<dyn Error>> {
    let mut cursor = std::io::Cursor::new(bytes);
    Ok(ash::util::read_spv(&mut cursor)?)
}

pub(crate) fn load_spv<T: AsRef<Path>>(path: T) -> Vec<u32> {

    let mut file = std::fs::File::open(path).unwrap();
    let mut text = Vec::new();
    file.read_to_end(&mut text).unwrap();

    assert_eq!(text.len() % 4, 0);
    assert_eq!(0x07230203, u32::from_le_bytes([text[0], text[1], text[2], text[3]]));

    read_shader_from_bytes(&text).unwrap()
}

