#[macro_use]
extern crate glium;
extern crate imgui;
extern crate cgmath;

mod support;
mod mesh;

#[allow(unused_imports)]
use glium::{glutin, Surface};
#[allow(unused_imports)]
use glium::index::PrimitiveType;
use support::{ System };
use mesh::Mesh;

struct App {
    mesh : Box<dyn mesh::MeshTrait>,
    running : bool,
}

impl App {
    pub fn new(system : &System) -> Self {
        let vertex_buffer = {
            #[derive(Copy, Clone)]
            struct Vertex {
                position: [f32; 2],
                color: [f32; 3],
                uv: [f32; 2],
            }

            implement_vertex!(Vertex, position, color, uv);

            glium::VertexBuffer::new(&system.display,
                &[
                Vertex { position: [-0.5, -0.5], color: [0.0, 1.0, 0.0], uv:[0.0, 0.0] },
                Vertex { position: [ 0.0,  0.5], color: [0.0, 0.0, 1.0], uv:[0.0, 0.0]},
                Vertex { position: [ 0.5, -0.5], color: [1.0, 0.0, 0.0], uv:[0.0, 0.0] },
                ]
            ).unwrap()
        };
        let index_buffer = glium::IndexBuffer::new(&system.display, PrimitiveType::TrianglesList,
            &[0u16, 1, 2]).unwrap();

        let mesh = Mesh::new(&system, vertex_buffer, index_buffer);

        Self {
            mesh : Box::new(mesh),
            running : true,
        }
    }

}

pub fn cos01(x: f64) -> f64 { (x.cos() / 2.0) + 0.5 }

impl support::App for App {

    fn draw(&self, frame_time : &support::FrameTime, frame : &mut glium::Frame) {
        let t = frame_time.now_as_duration().as_secs_f64();

        frame.clear_color(cos01(t * 10.0) as f32, 0.0, 0.0, 0.0);

        let m = cgmath::Matrix4::<f32>::from_scale(cos01(t) as f32);

        self.mesh.draw(m, frame);
    }

    fn handle_character(&mut self, c : char) {
        if c == 'q' {
            self.close_requested()
        }
    }

    fn close_requested(&mut self) {
        self.running = false;
    }

    fn update(&mut self, _frame_time : &support::FrameTime) {
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

fn main() {
    let mut system = System::new();
    let mut app = App::new(&system);
    system.run_app(&mut app);
}



