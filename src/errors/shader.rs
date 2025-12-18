use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Error create Shader")]
    ShaderCreationFailed(ash::vk::Result),
}
