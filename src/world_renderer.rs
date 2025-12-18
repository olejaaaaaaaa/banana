use winit::window;
use ash::vk;
use crate::{AttributeDescriptions, BindingDescriptions, Bindless, GraphicsPipelineBuilder, LayoutHandle, PassBuilder, PassContext, Pipeline, PipelineLayoutBuilder, RenderContext, RenderGraph, RenderGraphBuilder, RenderTarget, ResourceManager, Scene, SimpleRenderer, Vertex};


pub struct AABB {

}

pub struct Transforms {

}

pub struct UiRenderer {}


pub struct WorldRenderer {
    resources: ResourceManager,
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

    pub fn new(window: &winit::window::Window) -> Self {

        let ctx = RenderContext::new(&window).unwrap();
        let scene = Scene::new();
        let mut res = ResourceManager::new();

        let mut builder = RenderGraphBuilder::new();
        let simple = SimpleRenderer::new(&ctx, &mut res, &mut builder);
        let graph = builder.compile(&ctx);

        let ui = UiRenderer {};
        let bindless = Bindless {};
        let transforms = Transforms {};
        let aabb = AABB {};

        WorldRenderer { 
            resources: res,
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
        self.ctx.window.resize(&self.ctx.device, width, height);
    }

    pub fn draw_frame(&mut self) {
        self.graph.execute(&mut self.ctx, &self.scene);
    }
}