use log::info;
use crate::{App, AppBuilder, Device, DeviceBuilder, Fence, FenceBuilder, FrameBuffer, FrameBufferBuilder, Image, ImageBuilder, ImageView, ImageViewBuilder, Instance, InstanceBuilder, QueuePool, RenderPass, RenderPassBuilder, Semaphore, SemaphoreBuilder, Surface, SurfaceBuilder, Swapchain, SwapchainBuilder, VulkanResult};
use ash::vk;


pub struct FrameSync {
    pub image_available: Semaphore,
    pub render_finished: Semaphore,
    pub in_flight_fence: Fence
}

impl FrameSync {
    pub fn new(device: &Device) -> VulkanResult<FrameSync> {
        Ok(FrameSync { 
            image_available: SemaphoreBuilder::new(device).build()?,
            render_finished: SemaphoreBuilder::new(device).build()?,
            in_flight_fence: FenceBuilder::signaled(device).build()?
        })
    }
}
pub struct WindowManager {
    pub(crate) resolution: vk::Extent2D,
    pub(crate) frame_sync: Vec<FrameSync>,
    pub(crate) frame_buffers: Vec<FrameBuffer>,
    pub(crate) image_views: Vec<ImageView>,
    pub(crate) current_frame: usize,
    pub(crate) depth_image: Image,
    pub(crate) depth_view: ImageView,
    pub(crate) render_pass: RenderPass,
    pub(crate) surface: Surface,
    pub(crate) swapchain: Swapchain,
}

impl WindowManager {
    pub fn resize(&mut self, device: &GraphicsDevice, width: u32, height: u32) {

        info!("New size: {:?}", (width, height));

        unsafe {
            let _ = device.device_wait_idle();
        }

        let caps = self
            .surface
            .get_physical_device_surface_capabilities(&device.phys_dev);

        let swapchain = SwapchainBuilder::default(&device.instance, &device.device, &self.surface)
            .old_swapchain(self.swapchain.raw)
            .extent(caps.current_extent)
            .format(vk::Format::R8G8B8A8_SRGB)
            .build()
            .unwrap();

        let depth_image = ImageBuilder::depth(&device, vk::Format::D32_SFLOAT, caps.current_extent).build().unwrap();
        let depth_view = ImageViewBuilder::depth(&device, vk::Format::D32_SFLOAT, depth_image.raw).build().unwrap();

        let mut image_views = vec![];
        for i in swapchain.get_swapchain_images().unwrap() {
            let image_view = ImageViewBuilder::new_2d(&device, vk::Format::R8G8B8A8_SRGB, i).build().unwrap();
            image_views.push(image_view);
        }

        let mut frame_buffers = vec![];

        for i in &image_views {

            let frame_buffer = FrameBufferBuilder::new(&device, self.render_pass.raw)
                .add_attachment(i.raw)
                .add_attachment(depth_view.raw)
                .extent(caps.current_extent)
                .layers(1)
                .build()
                .unwrap();

            frame_buffers.push(frame_buffer);
        }
        
        self.depth_image.destory(&device);
        self.depth_image = depth_image;

        self.depth_view.destroy(&device);
        self.depth_view = depth_view;

        for i in &self.image_views {
            i.destroy(&device);
        }
        self.image_views = image_views;


        self.frame_buffers = frame_buffers;
        self.resolution = caps.current_extent;

        self.swapchain.destroy();
        self.swapchain = swapchain;
    }
}

pub struct GraphicsDevice {
    app: App,
    pub(crate) queue_pool: QueuePool,
    phys_dev: vk::PhysicalDevice,
    instance: Instance,
    pub(crate) device: Device,
}

impl std::ops::Deref for GraphicsDevice {
    type Target = Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}


pub struct RenderContext {
    pub(crate) window: WindowManager,
    pub(crate) device: GraphicsDevice,
}

impl RenderContext {
    pub fn new(window: &winit::window::Window) -> VulkanResult<Self> {

        let app = AppBuilder::default().build()?;
        let instance = InstanceBuilder::default(&app).build()?;
        let surface = SurfaceBuilder::new(&app, &instance, window).build()?;

        let phys_dev = unsafe { instance.raw.enumerate_physical_devices().unwrap() };
        let phys_dev = phys_dev[0];

        let device = DeviceBuilder::default(&instance, &phys_dev).build()?;

        let caps = surface.get_physical_device_surface_capabilities(&phys_dev);

        let swapchain = SwapchainBuilder::default(&instance, &device, &surface)
            .extent(caps.current_extent)
            .format(vk::Format::R8G8B8A8_SRGB)
            .build()?;

        let render_pass = RenderPassBuilder::default(&device, vk::Format::R8G8B8A8_SRGB, vk::Format::D32_SFLOAT).build()?;

        let depth_image = ImageBuilder::depth(&device, vk::Format::D32_SFLOAT, caps.current_extent).build()?;
        let depth_view = ImageViewBuilder::depth(&device, vk::Format::D32_SFLOAT, depth_image.raw).build()?;

        let mut image_views = vec![];

        for i in swapchain.get_swapchain_images().unwrap() {
            let image_view = ImageViewBuilder::new_2d(&device, vk::Format::R8G8B8A8_SRGB, i).build()?;
            image_views.push(image_view);
        }

        let mut frame_buffers = vec![];

        for i in &image_views {

            let frame_buffer = FrameBufferBuilder::new(&device, render_pass.raw)
                .add_attachment(i.raw)
                .add_attachment(depth_view.raw)
                .extent(caps.current_extent)
                .layers(1)
                .build()?;

            frame_buffers.push(frame_buffer);
        }

        let pool = QueuePool::new(&device.raw, &device.queue_family_props);
        let mut frame_sync = vec![];

        for _ in 0..frame_buffers.len() {
            frame_sync.push(FrameSync::new(&device)?);
        }

        Ok(Self {
            window: WindowManager { 
                resolution: caps.current_extent,
                frame_sync,
                image_views,
                depth_image: depth_image,
                frame_buffers,
                depth_view,
                current_frame: 0,
                surface, 
                swapchain, 
                render_pass 
            },
            device: GraphicsDevice {
                app,
                instance,
                device,
                queue_pool: pool,
                phys_dev,
            },
        })
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {

        self.window.swapchain.destroy();
        // self.device.device.destroy_render_pass(&self.window.render_pass);
        self.window.depth_image.destory(&self.device);
        self.window.depth_view.destroy(&self.device);
        
        for i in &self.window.frame_buffers {
            i.destroy(&self.device);
        }

        for i in &self.window.image_views {
            i.destroy(&self.device);
        }
        
        self.device.device.destroy();
        self.window.surface.destroy();
        self.device.instance.destroy();
    }
}
