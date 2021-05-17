use super::{ App, imgui, glutin, };
use super::frametime::FrameTime;
// use super::{glutin, imgui};
//
//
use glutin::event_loop::{ControlFlow, EventLoop };

use imgui_glium_renderer::Renderer;
use glutin::event::{ Event, WindowEvent };

use imgui::Context;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use glium::{ Display, Surface };
use std::time::Instant;

pub struct System {
    pub display: Display,
    pub renderer: Renderer,
    pub platform: WinitPlatform,
    pub event_loop: EventLoop<()>,
    pub frame_time: FrameTime,
    pub imgui: Context,
}

impl Default for System {
    fn default() -> Self {

        let event_loop = EventLoop::new();
        let wb = glutin::window::WindowBuilder::new();

        let cb = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);


        let display = Display::new(wb, cb, &event_loop).expect("Building display");

        let mut imgui = Context::create();

        imgui.set_ini_filename(Some("glht.ini".into()));

        let mut platform = WinitPlatform::init(&mut imgui);

        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
        }

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (18.0 * hidpi_factor) as f32;
        let rbytes = include_bytes!("../../resources/Inconsolata.otf");

        imgui.fonts().add_font(&[imgui::FontSource::TtfData {
            data: rbytes,
            size_pixels: font_size,
            config: Some(imgui::FontConfig {
                size_pixels: font_size,
                name: Some(String::from("Roboto")),
                // size_pixels: font_size,
                oversample_h: 4,
                oversample_v: 4,
                pixel_snap_h: true,
                ..imgui::FontConfig::default()
            }),
        }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        let renderer =
            Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        Self {
            display,
            renderer,
            platform,
            event_loop,
            frame_time: FrameTime::from_now(),
            imgui,
        }
    }

}

impl System {
    pub fn new() -> Self {
        Self::default()
    }

    //fn render(&mut self, app: &mut dyn App) {
    //    let platform = &mut self.platform;
    //    let gl_window = self.display.gl_window();
    //    let window = gl_window.window();
    //    // Drawing from here
    //    //
    //    let imgui = &mut self.imgui;
    //    let renderer = &mut self.renderer;

    //    platform
    //        .prepare_frame(imgui.io_mut(), window)
    //        .expect("Preparing frame start");

    //    let mut ui = imgui.frame();

    //    app.ui(&mut ui);

    //    platform.prepare_render(&ui, &window);

    //    let draw_data = ui.render();

    //    // let draw_data = ui.render();

    //    let mut frame = self.display.draw();

    //    app.draw(&mut frame);

    //    renderer
    //        .render(&mut frame, draw_data)
    //        .expect("Rendering failed");

    //    frame.finish().expect("Frame completion failed");
    //}

    // fn process(&mut self, app: & mut dyn App) {
    //     // let &mut io = self.imgui.io_mut();

    //     // self.frame_time.update();

    //     // let mut redraw = false;

    //     // let dt = self.frame_time;
    //     // let gl_window = self.display.gl_window();
    //     // let window = gl_window.window();

    //     // let platform = &mut self.platform;

    //     // let ev_loop = self.event_loop;

    //     // ev_loop.run(move |event, window_target , cflow| {
    //     //     // redraw = redraw || app.handle_event(&dt, &event);
    //     //     platform.handle_event(&mut io, &window, &event);
    //     // });

    //     // app.update(&dt);
    // }

    pub fn main_loop<F: App + 'static >(self , mut app : F) {
        let System {
            event_loop,
            display,
            mut imgui,
            mut platform,
            mut renderer,
            ..
        } = self;

        let mut last_frame = Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }

            Event::MainEventsCleared => {
                let gl_window = display.gl_window();
                platform
                    .prepare_frame(imgui.io_mut(), gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window.window().request_redraw();
            }

            Event::RedrawRequested(_) => {
                let mut ui = imgui.frame();

                let mut run = true;

                app.ui(&mut ui);

                if !run {
                    *control_flow = ControlFlow::Exit;
                }

                let gl_window = display.gl_window();
                let mut target = display.draw();
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
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

            event => {
                let gl_window = display.gl_window();
                app.handle_event(&event);
                platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
            }
        })
    }
}
