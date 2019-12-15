use imgui::{Context};

use imgui_winit_support::{HiDpiMode, WinitPlatform};
use glium::{glutin};
use imgui_glium_renderer::Renderer;
// use glium::{Surface};
//

use crate::frametime::FrameTime;

pub struct System {
    pub display : glium::Display,
    pub imgui_renderer : imgui_glium_renderer::Renderer,
    pub platform : WinitPlatform,
    pub event_loop : glutin::EventsLoop,
    pub frame_time : FrameTime,
    pub imgui : Context,
    font_id : imgui::FontId,
}

pub trait App {
    fn draw(&self, display : &mut glium::Frame);

    fn handle_event(&mut self, _frame_time : &FrameTime, input_event : glutin::Event) {
        use glutin::WindowEvent::*;
        use glutin::Event::WindowEvent;
        if let WindowEvent {event, ..} = input_event {
            match event {
                ReceivedCharacter(ch) => self.handle_character(ch),
                CloseRequested => self.close_requested(),
                Resized(..) => {
                    // self.mesh.draw(frame);
                },
                _ => (),
            }
        };
    }

    fn ui(&self, _ui : &mut imgui::Ui) {
    }

    fn update(&mut self, frame_time : &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, c : char);
}

impl System {
    pub fn new() -> Self {

        let event_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new();
        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let mut imgui = Context::create();

        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);

        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
        }

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (15.0 * hidpi_factor) as f32;
        let rbytes = include_bytes!("../resources/Roboto-Regular.ttf");

        let font_id = imgui.fonts().add_font(&[
            imgui::FontSource::TtfData {
                data: rbytes,
                size_pixels: font_size,
                config: Some(imgui::FontConfig {
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
            font_id,
        }
    }

    pub fn process(&mut self, app : &mut dyn App) {

        {
            let event_loop = &mut self.event_loop;
            let dt = &mut self.frame_time;

            let gl_window = self.display.gl_window();
            let window = gl_window.window();
            let platform = &mut self.platform;
            let ev_mut = self.imgui.io_mut();

            // Update frame time
            dt.update();

            // Handle all events
            event_loop.poll_events(|event| {
                platform.handle_event(ev_mut, &window, &event);
                app.handle_event(dt, event);
            });

            // Update the app
            app.update(dt);
        }


        {
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

            let font_id = ui.push_font(self.font_id);

            app.ui(&mut ui);

            font_id.pop(&ui);

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
    }

    pub fn run_app(&mut self, app: &mut dyn App) {
        while app.is_running() {
            self.process(app);
        }
    }
}

pub fn make_shaders(display : &glium::Display) -> glium::Program {

    program!(display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec3 color;
                out vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;
                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
        },

        110 => {
            vertex: "
                #version 110
                uniform mat4 matrix;
                attribute vec2 position;
                attribute vec3 color;
                varying vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 110
                varying vec3 vColor;
                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            ",
        },

        100 => {
            vertex: "
                #version 100
                uniform lowp mat4 matrix;
                attribute lowp vec2 position;
                attribute lowp vec3 color;
                varying lowp vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

            fragment: "
                #version 100
                varying lowp vec3 vColor;
                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            ",
        },
        ).unwrap()
}

