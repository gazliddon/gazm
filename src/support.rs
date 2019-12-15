use imgui::{Context};

use imgui_winit_support::{HiDpiMode, WinitPlatform};
use glium::{glutin};
use imgui_glium_renderer::Renderer;
// use glium::{Surface};


////////////////////////////////////////////////////////////////////////////////

use std::time::{ Duration, Instant };

pub struct FrameTime {
    base_time : Instant,
    last_sync : Instant,
    last_dt   : Duration,
}

impl FrameTime {
    pub fn from_now() -> Self {
        Self {
            base_time : Instant::now(),
            last_sync : Instant::now(),
            last_dt : Duration::new(0,0),
        }
    }

    pub fn now_as_duration(&self) -> Duration {
        self.last_sync - self.base_time
    }

    pub fn dt(&self) -> f64 {
        self.last_dt.as_secs_f64()
    }

    pub fn update(&mut self)  {
        let now = Instant::now();
        let dt = now - self.last_sync;
        self.last_dt = dt;
        self.last_sync = now;
    }

}

////////////////////////////////////////////////////////////////////////////////

pub struct System {
    pub display : glium::Display,
    pub imgui_renderer : imgui_glium_renderer::Renderer,
    pub platform : WinitPlatform,
    pub event_loop : glutin::EventsLoop,
    pub frame_time : FrameTime,
    pub imgui : Context,
}

pub trait App {
    fn draw(&self, frame_time : &FrameTime, display : &mut glium::Frame);

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

    fn ui(&mut self, ui : &mut imgui::Ui) {

        use imgui::*;

        Window::new(im_str!("Hello world"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("こんにちは世界！"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();

                let mouse_pos = ui.io().mouse_pos;

                ui.text(format!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos[0], mouse_pos[1]
                ));
            });
    }

    fn update(&mut self, frame_time : &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, c : char);
}

impl System {
    pub fn new() -> Self {
        println!("Starting....");

        let event_loop = glutin::EventsLoop::new();
        let wb = glutin::WindowBuilder::new();
        let cb = glutin::ContextBuilder::new();
        let display = glium::Display::new(wb, cb, &event_loop).unwrap();

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        let imgui_renderer = Renderer::init(&mut imgui, &display).expect("Failed to initialize renderer");

        let mut platform = WinitPlatform::init(&mut imgui);

        {
            let gl_window = display.gl_window();
            let window = gl_window.window();
            platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
        }

        Self {
            display,
            imgui_renderer,
            platform,
            event_loop,
            frame_time : FrameTime::from_now(),
            imgui
        }
    }

    pub fn process(&mut self, app : &mut dyn App) {

        let event_loop = &mut self.event_loop;

        self.frame_time.update();

        let dt = &self.frame_time;

        let gl_window = self.display.gl_window();
        let window = gl_window.window();
        let io_mut = self.imgui.io_mut();
        let platform = &mut self.platform;

        event_loop.poll_events(|event| {
            platform.handle_event(io_mut, &window, &event);
            app.handle_event(dt, event);
        });

        let io = self.imgui.io_mut();
        platform.prepare_frame(io, &window).unwrap();

        let mut ui = self.imgui.frame();

        app.update(dt);
        app.ui(&mut ui);

        let mut frame = self.display.draw();

        self.platform.prepare_render(&ui, &window);

        let draw_data = ui.render();

        app.draw(dt, &mut frame);

        self.imgui_renderer
            .render(&mut frame, draw_data)
            .expect("Rendering failed");

        frame.finish().unwrap();
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

