
use vk_mem::Allocation;
use ash::vk;

pub struct GpuBuffer {
    buffer: vk::Buffer,
    allocation: Allocation
}

struct GpuBufferBuilder {
    
}