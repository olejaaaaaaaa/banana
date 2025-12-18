
use std::sync::Arc;

use ash::vk;
use slotmap::{SlotMap, new_key_type};

new_key_type! {
    pub struct FrameBufferHandle;
}

use crate::{CommandPoolBuilder, Device, FrameBuffer, GraphicsPipeline, LayoutHandle, RenderContext, Renderable, Scene};
type Execute = dyn Fn(&PassContext, &[Renderable]);
type Setup = dyn Fn();



pub struct PassContext<'a> {
    resolution: vk::Extent2D,
    device: &'a Device,
    resources: Arc<RenderGraphResources>,
    cmd: vk::CommandBuffer,
    pipeline: Option<vk::Pipeline>,
    layout: Option<LayoutHandle>
}

impl<'a> PassContext<'a> {

    pub fn bind_pipeline(&self) {

        let pipeline = self.pipeline.unwrap();

        unsafe {

            self.device.cmd_bind_pipeline(
                self.cmd, 
                vk::PipelineBindPoint::GRAPHICS, 
                pipeline
            );

            let viewport = vk::Viewport::default()
                .height(self.resolution.height as f32)
                .width(self.resolution.width as f32)
                .x(0.0)
                .y(0.0)
                .max_depth(1.0)
                .min_depth(0.0);

            self.device.cmd_set_viewport(self.cmd, 0, &[viewport]);

            let scissor = vk::Rect2D::default()
                .extent(self.resolution)
                .offset(vk::Offset2D { x: 0, y: 0 });

            self.device.cmd_set_scissor(self.cmd, 0, &[scissor]);
        }
    }

    pub fn draw(&self, vertex_count: u32) {
        unsafe {
            self.device.cmd_draw(self.cmd, vertex_count, 1, 0, 0);
        }
    }
}

pub struct FrameDesc {
    pub width: u32,
    pub height: u32,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags
}

pub struct RenderGraphResources {
    frame_buffer: SlotMap<FrameBufferHandle, FrameBuffer>
}

impl RenderGraphResources {
    pub fn new() -> Self {
        RenderGraphResources { frame_buffer: SlotMap::with_key() }
    }
}

pub struct RenderGraph {
    queue: vk::Queue,
    resources: Arc<RenderGraphResources>,
    passes: Vec<Pass>,
    cmd_bufs: Vec<Vec<vk::CommandBuffer>>
}

impl RenderGraph {
    pub fn execute(&self, ctx: &mut RenderContext, scene: &Scene) {

        let window = &mut ctx.window;
        let sync = &window.frame_sync[window.current_frame % window.frame_buffers.len()];
        let device = &ctx.device.device;

        unsafe {
            device.wait_for_fences(&[sync.in_flight_fence.raw], true, u64::MAX).expect("Error wait for fences");
            device.reset_fences(&[sync.in_flight_fence.raw]).expect("Error wait for fences");
        }

        let (image_index, _) = unsafe { 
            window.swapchain.loader.acquire_next_image(
                window.swapchain.raw, 
                u64::MAX, 
                sync.image_available.raw, 
                vk::Fence::null()
            ).unwrap()
        };

        let buffers = &self.cmd_bufs[image_index as usize];

        for &buffer in buffers {
            unsafe { 
                device.reset_command_buffer(buffer, vk::CommandBufferResetFlags::empty()).expect("Error reset command buffer") 
            }
        }

        for index in 0..self.passes.len() {

            let pass = &self.passes[index];
            let cbuf = buffers[index];

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

            unsafe { 
                device.begin_command_buffer(cbuf, &begin_info).expect("Error begin command buffer") 
            };

            let clear_values = [
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [5.0/255.0, 5.0/255.0, 5.0/255.0, 1.0],
                    },
                },
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    },
                },
            ];

            let frame_buffer = &window.frame_buffers[image_index as usize];
            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(window.render_pass.raw)
                .framebuffer(frame_buffer.raw)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: window.resolution,
                })
                .clear_values(&clear_values);

                unsafe { device.cmd_begin_render_pass(
                    cbuf,
                    &render_pass_begin_info,
                vk::SubpassContents::INLINE,
                )};

            {
                let pass_ctx = PassContext { 
                    device,
                    resolution: window.resolution,
                    resources: self.resources.clone(),
                    cmd: cbuf, 
                    pipeline: Some(pass.pipeline.raw()), 
                    layout: Some(pass.layout), 
                };

                let nope = vec![];
                let renderables = scene.renderables.get(&pass.name).unwrap_or(&nope);
                (pass.execute)(&pass_ctx, &renderables);
            }

            unsafe { 
                device.cmd_end_render_pass(cbuf);
                let _ = device.end_command_buffer(cbuf);
            };

            let sync = &window.frame_sync[window.current_frame % window.frame_buffers.len()];
            let wait_semaphores = [sync.image_available.raw];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [sync.render_finished.raw];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&buffers)
                .signal_semaphores(&signal_semaphores);

            unsafe { device.queue_submit(self.queue, &[submit_info], sync.in_flight_fence.raw).expect("Error submit commands to queue") };

            let binding1 = [window.swapchain.raw];
            let binding = [image_index];

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&binding1)
                .image_indices(&binding);

            let _ = unsafe { window.swapchain.loader.queue_present(self.queue, &present_info) } ;
            window.current_frame += 1;
        }
    }
}


pub struct RenderGraphBuilder {
    frame_buffer: SlotMap<FrameBufferHandle, FrameDesc>,
    passes: Vec<Pass>
}


impl RenderGraphBuilder {

    pub fn new() -> Self {
        RenderGraphBuilder { 
            passes: vec![], 
            frame_buffer: SlotMap::with_key(),
        }
    }

    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }

    pub fn create_framebuffer(&mut self, desc: FrameDesc) -> FrameBufferHandle {
        self.frame_buffer.insert(desc)
    }

    pub fn compile(self, ctx: &RenderContext) -> RenderGraph {

        let mut res= RenderGraphResources::new();

        for i in self.frame_buffer {
            
        }

        let mut cmd_bufs = Vec::with_capacity(ctx.window.frame_buffers.len());
        let pool = CommandPoolBuilder::reset(&ctx.device).build().unwrap();

        for _ in 0..ctx.window.frame_buffers.len() {
            let buffers = pool.create_command_buffers(&ctx.device, self.passes.len() as u32).unwrap();
            cmd_bufs.push(buffers);
        }

        let queue = ctx.device.queue_pool.get_queue(vk::QueueFlags::GRAPHICS).unwrap();

        RenderGraph {  
            queue,
            cmd_bufs,
            passes: self.passes,
            resources: Arc::new(res)
        }
    }
}

pub enum RenderTarget {
    Swapchain,
    FrameBuffer,
}

pub enum Pipeline {
    Graphics(GraphicsPipeline),
    Compute()
}

impl Pipeline {
    pub fn raw(&self) -> vk::Pipeline {
        match self {
            Pipeline::Graphics(pipe ) => {
                pipe.raw
            }
            _ => panic!("AAAA")
        }
    }
}

pub struct PassBuilder {
    name: String,
    target: RenderTarget,
    execute: Option<Box<Execute>>,
    pipeline: Option<Pipeline>,
    pipeline_layout: Option<LayoutHandle>
}

impl PassBuilder {
    pub fn new<S: Into<String>>(name: S) -> Self {
        PassBuilder { 
            name: name.into(), 
            target: RenderTarget::Swapchain,
            execute: None, 
            pipeline: None, 
            pipeline_layout: None 
        }
    }

    pub fn target(mut self, target: RenderTarget) -> Self {
        self.target = target;
        self
    }

    pub fn execute(mut self, execute: Box<Execute>) -> Self {
        self.execute = Some(execute);
        self
    }

    pub fn use_pipeline(mut self, pipeline: Pipeline, layout: LayoutHandle) -> Self {
        self.pipeline = Some(pipeline);
        self.pipeline_layout = Some(layout);
        self
    }

    pub fn build(self) -> Pass {
        Pass {  
            target: self.target,
            name: self.name,
            pipeline: self.pipeline.unwrap(),
            layout: self.pipeline_layout.unwrap(),
            execute: self.execute.unwrap()
        }
    }
}

pub struct Pass {
    name: String,
    target: RenderTarget,
    pipeline: Pipeline,
    layout: LayoutHandle,
    execute: Box<Execute>
}

