use std::collections::HashMap;

use crate::{GameObject, Material, Renderable};




#[repr(C)]
struct Camera {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
    pos:  [f32; 4]
}



pub struct Scene {
    pub camera: Camera,
    pub game_objects: Vec<GameObject>,
    pub renderables: HashMap<String, Vec<Renderable>>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { camera: 
            Camera { 
                view: [
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ], 
                proj: [
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                ], 
                pos: [0.0, 0.0, 0.0, 0.0]
            }, 
            game_objects: vec![], 
            renderables: HashMap::new()
        }
    }
}