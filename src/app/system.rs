use super::{ app::App, frametime::FrameTime };

use imgui::{Context};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use glium::{glutin};
use imgui_glium_renderer::Renderer;

pub struct System {
    pub display : glium::Display,
    pub imgui_renderer : imgui_glium_renderer::Renderer,
    pub platform : WinitPlatform,
    pub event_loop : glutin::EventsLoop,
    pub frame_time : FrameTime,
    pub imgui : Context,
}

impl System {
    pub fn new() -> Self {

        let event_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new();
    
        let cb = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);

        let display = glium::Display::new(wb, cb, &event_loop)
            .expect("Building display");

        let mut imgui = Context::create();

        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);

        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
        }

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (20.0 * hidpi_factor) as f32;
        let rbytes = include_bytes!("../../resources/Roboto-Regular.ttf");

        imgui.fonts().add_font(&[
            imgui::FontSource::TtfData {
                data: rbytes,
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
                    size_pixels: font_size,
                    name: Some(String::from("Roboto")),
                    // size_pixels: font_size,
                    oversample_h: 4,
                    oversample_v: 4,
                    pixel_snap_h : true,
                    ..imgui::FontConfig::default()
                }),
            },
            ]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        let imgui_renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        Self {
            display,
            imgui_renderer,
            platform,
            event_loop,
            frame_time : FrameTime::from_now(),
            imgui,
        }
    }

    fn render(&mut self, app: &dyn App) {

        let platform = &mut self.platform;
        let gl_window = self.display.gl_window();
        let window = gl_window.window();
        // Drawing from here
        //
        let imgui = &mut self.imgui;
        let renderer = &mut self.imgui_renderer;

        platform
            .prepare_frame(imgui.io_mut(), &window)
            .expect("Preparing frame start");

        let mut ui = imgui.frame();

        app.ui(&mut ui);

        platform.prepare_render(&ui, &window);

        let draw_data = ui.render();

        // let draw_data = ui.render();

        let mut frame = self.display.draw();

        app.draw(&mut frame);

        renderer
            .render(&mut frame, draw_data)
            .expect("Rendering failed");

        frame
            .finish()
            .expect("Frame completion failed");
    }

    fn process(&mut self, app : &mut dyn App) {

        let io = self.imgui.io_mut();
        let dt = &mut self.frame_time;

        dt.update();
        {
            let event_loop = &mut self.event_loop;

            let gl_window = self.display.gl_window();
            let window = gl_window.window();
            let platform = &mut self.platform;

            // Update frame time

            // Handle all events
            event_loop.poll_events(|event| {
                platform.handle_event(io, &window, &event);
                app.handle_event(dt, event);
            });
            // Update the app
        }

        app.update(dt);
    }

    pub fn run_app(&mut self, app: &mut dyn App) {
        while app.is_running() {
            self.process(app);
            self.render(app);
        }
    }
}

