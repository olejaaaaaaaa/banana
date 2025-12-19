use std::sync::Arc;

use winit::window;
use ash::vk;
use crate::{AABB, AttributeDescriptions, BindingDescriptions, Bindless, DescriptorManager, FinalRenderer, FinalRendererBuilder, GraphicsPipelineBuilder, GridRenderer, LayoutHandle, PassBuilder, PassContext, Pipeline, PipelineLayoutBuilder, RenderContext, RenderGraph, RenderGraphBuilder, RenderTarget, ResourceManager, Scene, SimpleRenderer, Transforms, UiRenderer, Vertex};


pub struct GlobalUniforms {
    resolution: [u32; 2],
    mouse_pos: [f32; 2]
}


pub struct WorldRenderer {
    descriptors: DescriptorManager,
    resources: Arc<ResourceManager>,
    simple: SimpleRenderer,
    grid: GridRenderer,
    finall: FinalRenderer,
    ui: UiRenderer,
    bindless: Bindless,
    transforms: Transforms,
    aabb: AABB,
    graph: RenderGraph,
    scene: Scene,
    ctx: RenderContext,
}

impl WorldRenderer {

    pub fn new(window: &winit::window::Window) -> Self {

        let ctx = RenderContext::new(&window).unwrap();
        let scene = Scene::new();

        let mut res = ResourceManager::new();
        let mut builder = RenderGraphBuilder::new();

        let bindless = Bindless {};
        let transforms = Transforms {};
        let aabb = AABB {};

        let simple = SimpleRenderer::new(&ctx, &mut res, &mut builder, true);
        let grid = GridRenderer::new(&ctx, &mut res, &mut builder, true);
        let ui = UiRenderer {};

        let finall = FinalRendererBuilder::new(&ctx, &mut res, &mut builder)
            .with(0, simple.frame_buffer.unwrap())
            .with(1, grid.frame_buffer.unwrap())
            .build()
            .unwrap();

        let desc = DescriptorManager::new(&ctx.device).unwrap();
        let graph = builder.compile(&ctx, &desc);

        WorldRenderer { 
            descriptors: desc,
            resources: res.into(),
            grid,
            finall,
            simple, 
            ui, 
            bindless, 
            transforms, 
            aabb, 
            graph, 
            scene, 
            ctx
        }
    }

    pub fn reszie(&mut self, width: u32, height: u32) {

        // self.ctx.window.resize(&self.ctx.device, width, height);

        // let mut builder = RenderGraphBuilder::new();
        // let mut res = ResourceManager::new();

        // let simple = SimpleRenderer::new(&self.ctx, &mut res, &mut builder, true);
        // let finall = FinalRendererBuilder::new(&self.ctx, &mut res, &mut builder)
        //     .with(0, simple.frame_buffer.unwrap())
        //     .build()
        //     .expect("Error create FinalRenderer");

        // self.simple = simple;
        // self.finall = Some(finall);
        // self.resources = res.into();
        // self.graph = builder.compile(&self.ctx, &self.descriptors);

    }

    pub fn draw_frame(&mut self) {
       self.graph.execute(&mut self.ctx, &self.scene, self.resources.clone());
    }
}