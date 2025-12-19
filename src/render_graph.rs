
use std::sync::Arc;

use ash::vk;
use slotmap::{SecondaryMap, SlotMap, new_key_type};

new_key_type! {
    pub struct FrameBufferHandle;
}

new_key_type! {
    pub struct DescriptorSetHandle;
}

use crate::{CommandPool, DescriptorManager, DescriptorSetLayout, FrameBufferBuilder, Image, ImageBuilder, ImageView, ImageViewBuilder, RenderContext, Renderable, Sampler, SamplerBuilder, Scene, resources::*};
use crate::core::{CommandPoolBuilder, Device, FrameBuffer, GraphicsPipeline};

type Execute = dyn Fn(&PassContext, &[Renderable]);

pub struct PassContext<'a> {
    sets: Vec<vk::DescriptorSet>,
    resolution: vk::Extent2D,
    device: &'a Device,
    resources: Arc<RenderGraphResources>,
    s: Arc<ResourceManager>,
    cmd: vk::CommandBuffer,
    pipeline: Option<vk::Pipeline>,
    layout: Option<LayoutHandle>
}

impl<'a> PassContext<'a> {

    pub fn bind_pipeline(&self) {

        let pipeline = self.pipeline.expect("Missing Pipeline");
        let layout = self.s.get_layout(self.layout.unwrap()).unwrap();

        unsafe {

            if !self.sets.is_empty() {
                self.device.cmd_bind_descriptor_sets(
                    self.cmd, 
                    vk::PipelineBindPoint::GRAPHICS, 
                    layout.raw, 
                    0, 
                    &self.sets, 
                    &[]
                );
            }

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

pub struct GraphFrameBuffer {
    frame: FrameBuffer,
    image_view: ImageView,
    sampler: Sampler,
    image: Image
}

pub struct RenderGraphResources {
    frame_buffer: SecondaryMap<FrameBufferHandle, GraphFrameBuffer>,
    set: SecondaryMap<DescriptorSetHandle, vk::DescriptorSet>
}

impl RenderGraphResources {
    pub fn new() -> Self {
        RenderGraphResources { 
            frame_buffer: SecondaryMap::new(),
            set: SecondaryMap::new()
        }
    }
}

pub struct RenderGraph {
    queue: vk::Queue,
    pool: CommandPool,
    resources: Arc<RenderGraphResources>,
    passes: Vec<Pass>,
    cmd_bufs: Vec<Vec<vk::CommandBuffer>>
}

impl RenderGraph {
    pub fn execute(&self, ctx: &mut RenderContext, scene: &Scene, s: Arc<ResourceManager>) {

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

            let frame_buffer = match pass.target {
                RenderTarget::FrameBuffer(handle) => {
                    &self.resources.frame_buffer.get(handle).expect("Not found Frame Buffer").frame
                }
                RenderTarget::Swapchain => {
                    &window.frame_buffers[image_index as usize]
                }
            };

            let render_pass_begin_info = vk::RenderPassBeginInfo::default()
                .render_pass(window.render_pass.raw)
                .framebuffer(frame_buffer.raw)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: window.resolution,
                })
                .clear_values(&clear_values);

            unsafe { 
                device.cmd_begin_render_pass(cbuf, &render_pass_begin_info,vk::SubpassContents::INLINE)
            };

            {

                let mut sets = vec![];

                for i in &pass.bind_sets {
                    let set = *self.resources.set.get(i.set_handle).expect("Not found DescriptorSet");
                    sets.push(set);
                }

                let pass_ctx = PassContext { 
                    device,
                    sets,
                    s: s.clone(),
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
            };

            match pass.target {
                RenderTarget::FrameBuffer(handle) => {

                    let frame_buffer = self.resources.frame_buffer.get(handle).expect("Frame Buffer not found");

                    let image_barrier = vk::ImageMemoryBarrier::default()
                        .old_layout(vk::ImageLayout::PRESENT_SRC_KHR) 
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .image(frame_buffer.image.raw)
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        });

                    unsafe { 
                        device.cmd_pipeline_barrier(
                        cbuf,
                        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[image_barrier]
                        )
                    };
                }
                _ => {}
            }

            unsafe {
                let _ = device.end_command_buffer(cbuf);
            }
        }

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


pub struct RenderGraphBuilder {
    frame_buffer: SlotMap<FrameBufferHandle, FrameDesc>,
    set_layout: SlotMap<DescriptorSetHandle, DescriptorSetLayout>,
    binds: Vec<Binding>,
    passes: Vec<Pass>
}


pub struct BindSet {
    pub set: u32,
    pub set_handle: DescriptorSetHandle,
}

pub struct Binding {
    bind: u32,
    set: DescriptorSetHandle,
    frame: FrameBufferHandle
}

impl RenderGraphBuilder {

    pub fn new() -> Self {
        RenderGraphBuilder { 
            passes: vec![], 
            binds: vec![],
            set_layout: SlotMap::with_key(),
            frame_buffer: SlotMap::with_key(),
        }
    }

    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }

    pub fn create_frame_buffer(&mut self, desc: FrameDesc) -> FrameBufferHandle {
        self.frame_buffer.insert(desc)
    }

    pub fn create_descriptor_set(&mut self, set_layout: DescriptorSetLayout) -> DescriptorSetHandle {
        self.set_layout.insert(set_layout)
    }

    pub fn bind_resource_to_set(&mut self, bind: u32, set: DescriptorSetHandle, frame: FrameBufferHandle) {
         self.binds.push(Binding { 
            bind, 
            set, 
            frame 
        });
    }

    pub fn compile(self, ctx: &RenderContext, desc: &DescriptorManager) -> RenderGraph {

        let mut res= RenderGraphResources::new();

        for (handle, desc) in self.frame_buffer {
            
            let image = ImageBuilder::new_2d(
                &ctx.device, 
                desc.format, 
                vk::Extent2D { width: desc.width, height: desc.height }
            )
            .usage(desc.usage)
            .build()
            .unwrap();

            let image_view = ImageViewBuilder::new_2d(
                &ctx.device, 
                desc.format, 
                image.raw
            )
            .build()
            .unwrap();

            let frame_buffer = FrameBufferBuilder::new(&ctx.device, ctx.window.render_pass.raw)
                .add_attachment(image_view.raw)
                .add_attachment(ctx.window.depth_view.raw)
                .extent(ctx.window.resolution)
                .layers(1)
                .build()
                .unwrap();

            let sampler = SamplerBuilder::default(&ctx.device).build().unwrap();

            let frame = GraphFrameBuffer {
                frame: frame_buffer,
                sampler,
                image_view,
                image
            };

            res.frame_buffer.insert(handle, frame);

        }

        let mut cmd_bufs = Vec::with_capacity(ctx.window.frame_buffers.len());
        let pool = CommandPoolBuilder::reset(&ctx.device).build().unwrap();

        for _ in 0..ctx.window.frame_buffers.len() {
            let buffers = pool.create_command_buffers(&ctx.device, self.passes.len() as u32).unwrap();
            cmd_bufs.push(buffers);
        }

        for (handle, layout) in self.set_layout {
            let set = desc.create_descriptor_set(&ctx.device, &[layout.raw])[0];
            res.set.insert(handle, set);
        }

        for i in self.binds {
            
            let frame_buffer = res.frame_buffer.get(i.frame).expect("Not found Frame Buffer");

            let image_info = vk::DescriptorImageInfo::default()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(frame_buffer.image_view.raw)
                .sampler(frame_buffer.sampler.raw);

                let set = res.set.get(i.set).unwrap();

                let bind = &[image_info];

                let write = vk::WriteDescriptorSet::default()
                    .dst_set(*set)
                    .dst_binding(i.bind)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(bind);

                unsafe {
                    ctx.device.raw.update_descriptor_sets(&[write], &[]);
                }
        }

        let queue = ctx.device.queue_pool.get_queue(vk::QueueFlags::GRAPHICS).unwrap();

        RenderGraph {  
            queue,
            pool,
            cmd_bufs,
            passes: self.passes,
            resources: Arc::new(res)
        }
    }
}

pub enum RenderTarget {
    Swapchain,
    FrameBuffer(FrameBufferHandle),
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
    bind_sets: Vec<BindSet>,
    pipeline_layout: Option<LayoutHandle>
}

impl PassBuilder {
    pub fn new<S: Into<String>>(name: S) -> Self {
        PassBuilder { 
            bind_sets: vec![],
            name: name.into(), 
            target: RenderTarget::Swapchain,
            execute: None, 
            pipeline: None, 
            pipeline_layout: None 
        }
    }

    pub fn bind_descriptor_set(mut self, set: u32, set_handle: DescriptorSetHandle) -> Self {
        self.bind_sets.push(BindSet { set, set_handle });
        self
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
            bind_sets: self.bind_sets,
            pipeline: self.pipeline.unwrap(),
            layout: self.pipeline_layout.unwrap(),
            execute: self.execute.unwrap()
        }
    }
}

pub struct Pass {
    name: String,
    bind_sets: Vec<BindSet>,
    target: RenderTarget,
    pipeline: Pipeline,
    layout: LayoutHandle,
    execute: Box<Execute>
}

