#[allow(unused_imports)]
extern crate glutin;

#[macro_use] extern crate imgui_glium_renderer;

pub use imgui_glium_renderer::glium;
pub use imgui_glium_renderer::imgui;

#[macro_use] extern crate log;

#[allow(unused_imports)]
#[macro_use] extern crate serde_derive;

#[allow(dead_code)] pub mod colour;
#[allow(dead_code)] pub mod app;
#[allow(dead_code)] pub mod window;
#[allow(dead_code)] pub mod sourcewin;
#[allow(dead_code)] pub mod mesh;
#[allow(dead_code)] pub mod simple;
#[allow(dead_code)] pub mod dbgwin;
#[allow(dead_code)] pub mod textscreen;
#[allow(dead_code)] pub mod events;
#[allow(dead_code)] mod styles;

#[cfg(test)] mod tests;

