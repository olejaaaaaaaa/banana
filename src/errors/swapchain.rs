use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwapchainError {
    #[error("Error create Swapchain")]
    SwapchainCreationFailed(ash::vk::Result),
}
