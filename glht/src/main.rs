extern crate env_logger;

#[macro_use]
extern crate glium;
#[macro_use]
extern crate log;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

extern crate emu;

mod app;
mod mesh;
mod simple;

use app::{frametime::FrameTime, system::System, App};
use glium::index::PrimitiveType;
use glium::Surface;
use mesh::Mesh;

#[allow(dead_code)]
struct MyApp {
    mesh: Box<dyn mesh::MeshTrait>,
    running: bool,
    frame_time: FrameTime,
    machine: simple::simplecore::Simple,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
    uv: [f32; 2],
}

fn make_mesh(system: &System) -> Box<Mesh<Vertex, u16>> {
    let vertex_buffer = {
        implement_vertex!(Vertex, position, color, uv);

        glium::VertexBuffer::new(
            &system.display,
            &[
                Vertex {
                    position: [-0.5, -0.5],
                    color: [0.0, 1.0, 0.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    position: [0.0, 0.5],
                    color: [0.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    position: [0.5, -0.5],
                    color: [1.0, 0.0, 0.0],
                    uv: [0.0, 0.0],
                },
            ],
        )
        .unwrap()
    };
    let index_buffer =
        glium::IndexBuffer::new(&system.display, PrimitiveType::TrianglesList, &[0u16, 1, 2])
            .unwrap();

    Box::new(Mesh::new(&system, vertex_buffer, index_buffer))
}

impl MyApp {
    pub fn new(system: &System) -> Self {
        use simple::simplecore::Simple;

        let machine = Simple::new();
        let mesh = make_mesh(&system);

        Self {
            machine,
            mesh,
            running: true,
            frame_time: FrameTime::default(),
        }
    }
}

pub fn cos01(x: f64) -> f64 {
    (x.cos() / 2.0) + 0.5
}
pub fn sin01(x: f64) -> f64 {
    (x.sin() / 2.0) + 0.5
}

impl App for MyApp {
    fn draw(&self, frame: &mut glium::Frame) {
        use cgmath::*;

        let t = self.frame_time.now_as_duration().as_secs_f64();
        frame.clear_color(cos01(t * 10.0) as f32, 0.0, 0.0, 0.0);
        let m = Matrix4::<f32>::from_scale(cos01(t) as f32);
        self.mesh.draw(m, frame);
    }

    fn handle_character(&mut self, c: char) {
        if c == 'q' {
            self.close_requested()
        }

        if c == 's' {
            self.machine.step();
        }
    }

    fn close_requested(&mut self) {
        self.running = false;
    }

    fn update(&mut self, frame_time: &FrameTime) {
        self.frame_time = *frame_time;
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn ui(&self, ui: &mut imgui::Ui) {
        use imgui::*;

        Window::new(im_str!("Hello world"))
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(im_str!("Hello world!!!!!"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.text(im_str!("This....is...imgui-rs!"));
                ui.separator();

                let mouse_pos = ui.io().mouse_pos;

                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
            });
    }
}

fn main() {
    use std::env;
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut system = System::new();
    let mut app = MyApp::new(&system);
    system.run_app(&mut app);
}
