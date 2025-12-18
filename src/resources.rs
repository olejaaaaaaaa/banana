
use std::collections::HashMap;
use slotmap::*;
use crate::PipelineLayout;

new_key_type! { pub struct LayoutHandle; }

pub struct ResourceManager {
    layout: SlotMap<LayoutHandle, PipelineLayout>,
    cache: Cache
}

impl ResourceManager {
    
    pub fn new() -> Self {
        ResourceManager { 
            layout: SlotMap::with_key(),
            cache: Cache { layout: HashMap::new() } 
        }
    }

    pub fn add_layout(&mut self, layout: PipelineLayout) -> LayoutHandle {
        self.layout.insert(layout)
    }

    pub fn cache_layout<S: Into<String>>(&mut self, name: S, layout: PipelineLayout) -> LayoutHandle {
        let handle = self.layout.insert(layout);
        self.cache.layout.insert(name.into(), handle);
        handle
    }

}


pub struct Cache {
    layout: HashMap<String, LayoutHandle>
}