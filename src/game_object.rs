use std::{any::Any, sync::Weak};

#[derive(Clone, Copy)]
struct PushConstants {

}

#[derive(Clone)]
pub struct Material {
    pass: String,
    push: PushConstants
}

struct Transform {

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