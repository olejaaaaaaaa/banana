
use std::collections::HashMap;
use slotmap::*;
use crate::core::PipelineLayout;

new_key_type! { pub struct LayoutHandle; }
new_key_type! { pub struct SamplerHandle; }

pub struct ResourceManager {
    layout: SlotMap<LayoutHandle, PipelineLayout>,
    sampler: SlotMap<SamplerHandle, bool>,
    cache: Cache
}

impl ResourceManager {
    
    pub fn new() -> Self {
        ResourceManager { 
            sampler: SlotMap::with_key(),
            layout: SlotMap::with_key(),
            cache: Cache { layout: HashMap::new() } 
        }
    }

    pub fn add_layout(&mut self, layout: PipelineLayout) -> LayoutHandle {
        self.layout.insert(layout)
    }

    pub fn get_layout(&self, layout: LayoutHandle) -> Option<&PipelineLayout> {
        self.layout.get(layout)
    }

    pub fn get_layout_from_cache<S: Into<String>>(&mut self, name: S) -> Option<(&PipelineLayout, &LayoutHandle)> {
        let handle = self.cache.layout.get(&name.into());

        if let Some(handle) = handle {
            return Some((self.get_layout(*handle).unwrap(), handle));
        }
        
        None
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