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
}

impl FrameTime {
    pub fn from_now() -> Self {
        Self {
            base_time : Instant::now(),
            last_sync : Instant::now(),
        }
    }

    pub fn dt_as_duration(&self) -> Duration {
        self.last_sync - self.base_time
    }

    pub fn now_as_duration(&self) -> Duration {
        self.last_sync - self.base_time
    }

    pub fn dt(&self) -> f64 {
        self.dt_as_duration().as_secs_f64()
    }

    pub fn update(&mut self) -> f64 {
        self.last_sync = Instant::now();
        self.dt()
    }

}

////////////////////////////////////////////////////////////////////////////////

pub struct System {
    pub display : glium::Display,
    pub imgui_renderer : imgui_glium_renderer::Renderer,
    pub platform : WinitPlatform,
    pub event_loop : glutin::EventsLoop,
    pub frame_time : FrameTime,
}

pub trait App {
    fn draw(&self, frame_time : &FrameTime, display : &mut glium::Frame);
    fn handle_event(&mut self, frame_time : &FrameTime, event : glutin::Event);
    fn update(&mut self, frame_time : &FrameTime);
    fn is_running(&self) -> bool;
}

impl System {
    pub fn new() -> Self {
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
            frame_time : FrameTime::from_now()
        }
    }

    pub fn process(&mut self, app : &mut dyn App) {
        let event_loop = &mut self.event_loop;

        self.frame_time.update();

        let dt = &self.frame_time;

        event_loop.poll_events(|event| {
            app.handle_event(dt, event);
        });

        app.update(dt);

        let mut frame = self.display.draw();

        app.draw(dt, &mut frame);

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

