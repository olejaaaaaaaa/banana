use ash::vk;

use crate::{DescriptorSetLayoutBuilder, FrameBufferHandle, GraphicsPipelineBuilder, PassBuilder, PassContext, Pipeline, PipelineLayoutBuilder, RenderContext, RenderGraphBuilder, RenderTarget, ResourceManager, VulkanResult};



pub struct FinalRenderer {

}

pub struct FinalRendererBuilder<'a> {
    binds: Vec<(u32, FrameBufferHandle)>,
    res: &'a mut ResourceManager,
    ctx: &'a RenderContext,
    builder: &'a mut RenderGraphBuilder
}


impl<'a> FinalRendererBuilder<'a> {

    pub fn new(ctx: &'a RenderContext, res: &'a mut ResourceManager, builder: &'a mut RenderGraphBuilder) -> Self {
        Self { 
            ctx, 
            res,
            builder,
            binds: vec![]
        }
    }

    pub fn with(mut self, bind: u32, handle: FrameBufferHandle) -> Self {
        self.binds.push((bind, handle));
        self
    }
 
    pub fn build(self) -> VulkanResult<FinalRenderer> {

        let device = &self.ctx.device.device;
        let mut binds = vec![];

        for (bind, _) in &self.binds {
            binds.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(*bind)
                    .descriptor_count(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            );
        }

        let set_layout = DescriptorSetLayoutBuilder::new(device)
            .bindings(binds)
            .build()?;

        let color_blend = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A
            )
            .blend_enable(false);

        let layout = PipelineLayoutBuilder::new(&device)
            .set_layouts(vec![
                set_layout.raw
            ])
            .push_constant(vec![
                vk::PushConstantRange::default()
                    .offset(0)
                    .size(128)
                    .stage_flags(vk::ShaderStageFlags::VERTEX)
            ])
            .build()
            .unwrap();

        let set = self.builder.create_descriptor_set(set_layout);

        for (bind, frame) in &self.binds {
            self.builder.bind_resource_to_set(
                *bind, 
                set, 
                *frame
            );
        }
    
        let pipeline = GraphicsPipelineBuilder::new(&device)
            .vertex_shader_from_file(r"src\shared\shaders\spv\final-vert.spv")
            .fragment_shader_from_file(r"src\shared\shaders\spv\final-frag.spv")
            .render_pass(self.ctx.window.render_pass.raw)
            .pipeline_layout(layout.raw)
            .viewport(vec![
                vk::Viewport::default()
                    .x(0.0)
                    .y(0.0)
                    .width(self.ctx.window.resolution.width as f32)
                    .height(self.ctx.window.resolution.height as f32)
                    .min_depth(0.0)
                    .max_depth(1.0)
            ])
            .scissors(vec![
                vk::Rect2D::default()
                    .offset(vk::Offset2D { x: 0, y: 0 })
                    .extent(vk::Extent2D { 
                        width: self.ctx.window.resolution.width, 
                        height: self.ctx.window.resolution.height 
                    })
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
            .build()?;

        let layout = self.res.add_layout(layout);

        self.builder.add_pass(
            PassBuilder::new("Final Pass")
                .bind_descriptor_set(0, set)
                .use_pipeline(Pipeline::Graphics(pipeline), layout)
                .target(RenderTarget::Swapchain)
                .execute(Box::new(|ctx: &PassContext<'_>, _: &[crate::Renderable]| {
                    ctx.bind_pipeline();
                    ctx.draw(3);
                }))
                .build()
        );

        Ok(FinalRenderer {

        })
    }

}