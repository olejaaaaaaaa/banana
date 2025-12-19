

use ash::vk;

use crate::{AttributeDescriptions, BindingDescriptions, FrameBufferHandle, GraphicsPipelineBuilder, LayoutHandle, PassBuilder, PassContext, Pipeline, PipelineLayoutBuilder, RenderContext, RenderGraphBuilder, RenderTarget, ResourceManager, Vertex};


pub struct GridRenderer {
    pub frame_buffer: Option<FrameBufferHandle>,
    pub layout: LayoutHandle
}

impl GridRenderer {
    
    pub fn new(ctx: &RenderContext, res: &mut ResourceManager, builder: &mut RenderGraphBuilder, offscreen: bool) -> Self {

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

        let (layout, layout_handle) = res.get_layout_from_cache("Layout 1").unwrap();
    
        let pipeline = GraphicsPipelineBuilder::new(&ctx.device)
            .vertex_shader_from_file(r"src\shared\shaders\spv\grid-vert.spv")
            .fragment_shader_from_file(r"src\shared\shaders\spv\grid-frag.spv")
            .render_pass(ctx.window.render_pass.raw)
            .pipeline_layout(layout.raw)
            .viewport(vec![
                vk::Viewport::default()
                    .x(0.0)
                    .y(0.0)
                    .width(ctx.window.resolution.width as f32)
                    .height(ctx.window.resolution.height as f32)
                    .min_depth(0.0)
                    .max_depth(1.0)
            ])
            .scissors(vec![
                vk::Rect2D::default()
                    .offset(vk::Offset2D { x: 0, y: 0 })
                    .extent(vk::Extent2D { width: ctx.window.resolution.width, height: ctx.window.resolution.height })
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

        if offscreen {

            let frame_buffer = builder.create_frame_buffer(crate::FrameDesc { 
                width: ctx.window.resolution.width, 
                height: ctx.window.resolution.height, 
                format: vk::Format::R8G8B8A8_SRGB, 
                usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED
            });

            builder.add_pass(
                PassBuilder::new("Grid Pass")
                    .use_pipeline(Pipeline::Graphics(pipeline), *layout_handle)
                    .target(RenderTarget::FrameBuffer(frame_buffer))
                    .execute(Box::new(|ctx: &PassContext<'_>, renderables: &[crate::Renderable]| {
                        ctx.bind_pipeline();
                        ctx.draw(6);
                    }))
                    .build()
            );

            return Self { 
                frame_buffer: Some(frame_buffer), 
                layout: *layout_handle
            }
        } 

        builder.add_pass(
            PassBuilder::new("Grid Pass")
                .use_pipeline(Pipeline::Graphics(pipeline), *layout_handle)
                .target(RenderTarget::Swapchain)
                .execute(Box::new(|ctx: &PassContext<'_>, renderables: &[crate::Renderable]| {
                    ctx.bind_pipeline();
                    ctx.draw(6);
                }))
                .build()
        );

        Self { 
            frame_buffer: None, 
            layout: *layout_handle
        }
    }
}