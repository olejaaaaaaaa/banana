use std::{any::Any, sync::Weak};

#[repr(C)]
#[derive(Clone, Copy)]
struct PushConstants {
    transform_idx: u32
}

#[derive(Clone)]
pub struct Material {
    pass: String,
    push: PushConstants
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Transform {
    pub scale: [f32; 3]
}

pub struct GameObject {
    name: Option<String>,
    transform: Transform,
    components: Vec<Box<dyn Any>>,
    is_visible: bool,
    child: Option<Box<GameObject>>,
    parent: Option<Weak<Box<GameObject>>>
}

#[derive(Clone)]
pub struct Renderable {
    material: Material
}