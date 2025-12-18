use std::ffi::CStr;

use ash::vk;
use log::{debug, info, warn};
use winit::{
    event::KeyEvent, keyboard::PhysicalKey, monitor::VideoMode, raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle}, window::WindowAttributes
};

pub struct AABB {

}

pub struct Transforms {

}

pub struct UiRenderer {}

pub struct SimpleRenderer {

}

pub struct WorldRenderer {
    simple: SimpleRenderer,
    ui: UiRenderer,
    bindless: Bindless,
    transforms: Transforms,
    aabb: AABB,
    graph: RenderGraph,
    scene: Scene,
    ctx: RenderContext,
}

impl WorldRenderer {

    pub fn new() {

    }

    pub fn update(&self) {

    }

    pub fn draw(&self) {

    }
}

pub struct RenderContext {
    window: WindowManager,
    device: GraphicsDevice,
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

struct FrameSync {
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
struct WindowManager {
    resolution: vk::Extent2D,
    frame_sync: Vec<FrameSync>,
    frame_buffers: Vec<FrameBuffer>,
    image_views: Vec<ImageView>,
    current_frame: usize,
    depth_image: Image,
    depth_view: ImageView,
    render_pass: RenderPass,
    surface: Surface,
    swapchain: Swapchain,
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
    queue_pool: QueuePool,
    phys_dev: vk::PhysicalDevice,
    instance: Instance,
    device: Device,
}

impl std::ops::Deref for GraphicsDevice {
    type Target = Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

mod bindless;
pub use bindless::*;

mod queue_pool;
pub use queue_pool::*;

mod app;
pub use app::*;

mod instance;
pub use instance::*;

mod debug;
pub use debug::*;

mod surface;
pub use surface::*;

mod device;
pub use device::*;

mod swapchain;
pub use swapchain::*;

mod command_pool;
pub use command_pool::*;

mod frame_buffer;
pub use frame_buffer::*;

mod image;
pub use image::*;

mod phys_dev;
pub use phys_dev::*;

mod shader;
pub use shader::*;

mod pipeline_layout;
pub use pipeline_layout::*;

mod buffer;
pub use buffer::*;

mod subpass;
pub use subpass::*;

mod render_pass;
pub use render_pass::*;

mod image_view;
pub use image_view::*;

mod render_graph;
pub use render_graph::*;

mod errors;
pub use errors::*;

mod graphics_pipeline;
pub use graphics_pipeline::*;

mod resources;
pub use resources::*;

mod semaphore;
pub use semaphore::*;

mod scene;
pub use scene::*;

mod fence;
pub use fence::*;

mod game_object;
pub use game_object::*;

mod types;
pub use types::*;

fn main() {

    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let window = event_loop.create_window(
        WindowAttributes::new()
            .with_fullscreen(
                Some(
                    winit::window::Fullscreen::Borderless(None)
                )
            )
    ).unwrap();

    let mut ctx = RenderContext::new(&window).unwrap();
    let mut builder = RenderGraphBuilder::new();

    warn!("Init Context");
    
    let bind = Vertex::bind_desc();
    let attrs = Vertex::attr_desc();

    let color_blend = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(
            vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A
        )
        .blend_enable(false);

    let layout = PipelineLayoutBuilder::new(&ctx.device)
        .set_layouts(vec![])
        .push_constant(vec![
            vk::PushConstantRange::default()
                .offset(0)
                .size(128)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
        
        ])
        .build()
        .unwrap();
    
    let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
        .vertex_shader_from_file(r"src\shared\shaders\spv\base_simple-vert.spv")
        .fragment_shader_from_file(r"src\shared\shaders\spv\base_simple-frag.spv")
        .render_pass(ctx.window.render_pass.raw)
        .pipeline_layout(layout.raw)
        .viewport(vec![
            vk::Viewport::default()
                .x(0.0)
                .y(0.0)
                .width(800.0 as f32)
                .height(600.0 as f32)
                .min_depth(0.0)
                .max_depth(1.0)
        ])
        .scissors(vec![
            vk::Rect2D::default()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(vk::Extent2D { width: 800, height: 600 })
        ])
        .input_assembly(
            vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false)
        )
        .rasterization(
            vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::CLOCKWISE)
                .depth_bias_enable(false)
        )
        .vertex_input_info(
            vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&bind)
                .vertex_attribute_descriptions(&attrs)
        )
        .multisampling(
            vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        )
        .color_blending(
            vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .logic_op(vk::LogicOp::COPY)
                .attachments(&[color_blend])
        )
        .dynamic_state(vec![
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR
        ])
        .build()
        .unwrap();

    let mut res = ResourceManager::new();
    let layout = res.cache_layout("Layout 1", layout);

    builder.add_pass(
        PassBuilder::new("Simple Pass")
            .use_pipeline(Pipeline::Graphics(pipeline), layout)
            .target(RenderTarget::Swapchain)
            .execute(Box::new(|ctx, renderables| {
                ctx.bind_pipeline();
                ctx.draw(3);
            }))
            .build()
    );

    let graph = builder.compile(&ctx);
    let scene = Scene::new();

    event_loop
        .run(|ev, active_ev| match ev {

            winit::event::Event::AboutToWait => {
                window.request_redraw();
            }

            winit::event::Event::WindowEvent { window_id, event } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    active_ev.exit();
                },
                winit::event::WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                    match event.physical_key {
                        PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => {
                            active_ev.exit();
                        }
                        _ => {}
                    }
                },
                winit::event::WindowEvent::Resized(size) => {
                    let (width, height) = (size.width, size.height);
                    ctx.window.resize(&ctx.device, width, height);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    graph.execute(&mut ctx, &scene);
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
