use thiserror::Error;
use ash::vk;

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("Failed create SurfaceKHR (Vulkan error: {0:?})")]
    CreateSurface(vk::Result),
}