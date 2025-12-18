use std::ffi::CStr;

use ash::vk;
use log::{debug, info, warn};
use winit::{
    event::KeyEvent, keyboard::PhysicalKey, monitor::VideoMode, raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle}, window::WindowAttributes
};

mod world_renderer;
pub use world_renderer::*;

mod render_context;
pub use render_context::*;

mod game_object;
pub use game_object::*;

mod queue_pool;
pub use queue_pool::*;

mod bindless;
pub use bindless::*;

mod render_graph;
pub use render_graph::*;

mod resources;
pub use resources::*;

mod simple;
pub use simple::*;

mod scene;
pub use scene::*;

mod core;
pub use core::*;

fn main() {

    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
    let window = event_loop.create_window(
        WindowAttributes::new()
            .with_fullscreen(
                Some(
                    winit::window::Fullscreen::Borderless(None)
                )
            )
    ).unwrap();
    
    let mut world = WorldRenderer::new(&window);
    
    event_loop
        .run(|ev, active_ev| match ev {

            winit::event::Event::AboutToWait => {
                window.request_redraw();
            }

            winit::event::Event::WindowEvent { window_id, event } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    active_ev.exit();
                },
                winit::event::WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                    match event.physical_key {
                        PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => {
                            active_ev.exit();
                        }
                        _ => {}
                    }
                },
                winit::event::WindowEvent::Resized(size) => {
                    let (width, height) = (size.width, size.height);
                    world.reszie(width, height);
                }
                winit::event::WindowEvent::RedrawRequested => {
                    world.draw_frame();
                }
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}
