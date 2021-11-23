use super::frametime::FrameTime;
use super::{glutin, imgui, App};
// use super::{glutin, imgui};
//
//
use glutin::event_loop::{ControlFlow, EventLoop};

use glutin::event::{Event, WindowEvent, ModifiersState};
use imgui_glium_renderer::Renderer;

use crate::v2::*;
use glium::Display;
use imgui::Context;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

use lazy_static::*;

lazy_static! {

    static ref CODE_POINTS : Vec<u16> = {
        vec![
            0x2500,
            0x257f,
            0x0002,
            0x007f,
            0x0
        ]
    };


}

pub struct System {
    pub display: Display,
    pub renderer: Renderer,
    pub platform: WinitPlatform,
    pub event_loop: EventLoop<()>,
    pub frame_time: FrameTime,
    pub imgui: Context,
    scale : f64,
}

impl Default for System {
    fn default() -> Self {
        let event_loop = EventLoop::new();
        let wb = glutin::window::WindowBuilder::new();

        let cb = glutin::ContextBuilder::new()
            .with_double_buffer(Some(true))
            .with_multisampling(16)
            .with_hardware_acceleration(Some(true))
            .with_srgb(true)
            .with_vsync(true);

        let display = Display::new(wb, cb, &event_loop).expect("Building display");

        let mut imgui = Context::create();

        imgui.set_ini_filename(Some("glht.ini".into()));

        let mut platform = WinitPlatform::init(&mut imgui);

        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
        }

        let scale = platform.hidpi_factor();
        let font_size = (18.0 * scale) as f32;
        let rbytes = include_bytes!("../../resources/FiraCode-Retina.ttf");

        let range = imgui::FontGlyphRanges::from_slice(&CODE_POINTS);

        imgui.fonts().add_font(&[imgui::FontSource::TtfData {
            data: rbytes,
            size_pixels: font_size,
            config: Some(imgui::FontConfig {
                size_pixels: font_size,
                name: Some(String::from("Roboto")),
                oversample_h: 8,
                oversample_v: 8,
                pixel_snap_h: true,
                glyph_ranges : range,
                ..imgui::FontConfig::default()
            }),
        }]);

        imgui.io_mut().font_global_scale = (1.0 / scale) as f32;

        let renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        Self {
            display,
            renderer,
            platform,
            event_loop,
            frame_time: FrameTime::from_now(),
            imgui,
            scale,
        }
    }
}

impl System {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hidpi_factor(&self) -> f64 {
        self.platform.hidpi_factor()
    }

    pub fn to_logical(&self, v : &V2<f32>) -> V2<f32> {
        let sc = self.hidpi_factor() as f32;
        v / sc
    }

    pub fn main_loop<E, F: App<E> + 'static>(self, mut app: F) {
        let System {
            event_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
            mut frame_time,
            ..
        } = self;

        let mut last_frame = Instant::now();


        let mut mstate = ModifiersState::empty();
        let scale = self.scale;

        event_loop.run(move |event, _, control_flow| {
            
            if !app.is_running() {
                *control_flow = ControlFlow::Exit;
            }
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }

                Event::MainEventsCleared => {
                    frame_time.update();
                    app.update(&frame_time);

                    let gl_window = display.gl_window();
                    platform
                        .prepare_frame(imgui.io_mut(), gl_window.window())
                        .expect("Failed to prepare frame");
                    gl_window.window().request_redraw();
                }

                Event::RedrawRequested(_) => {
                    let mut ui = imgui.frame();

                    let gl_window = display.gl_window();
                    let mut target = display.draw();

                    // Get the inner size
                    // convert to logical pixels using hidpi
                    let dims = gl_window.window().inner_size().to_logical::<f64>(scale);

                    let pos = gl_window
                        .window()
                        .inner_position()
                        .unwrap_or_else(|_| glutin::dpi::PhysicalPosition::<i32>::new(0, 0))
                        .to_logical::<f64>(scale);

                    let dims = V2::new(dims.width, dims.height).as_usizes();
                    let pos = V2::new(pos.x, pos.y).as_isizes();

                    app.draw(scale, pos, dims, &mut target);
                    app.ui(scale, pos, dims, &mut ui);

                    platform.prepare_render(&ui, gl_window.window());

                    let draw_data = ui.render();

                    renderer
                        .render(&mut target, draw_data)
                        .expect("Rendering failed");

                    target.finish().expect("Failed to swap buffers");
                }

                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,

                Event::WindowEvent{ event: WindowEvent::ModifiersChanged(new_mstate), ..}  =>{
                    mstate = new_mstate;
                },

                event => {
                    let gl_window = display.gl_window();
                    app.handle_event(&event, mstate);

                    platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
                }
            }
        })
    }
}
