#[macro_use]
extern crate glium;
extern crate imgui;

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

impl support::App for App {

    fn draw(&self, _frame_time : &support::FrameTime, frame : &mut glium::Frame) {

        let r = _frame_time.now_as_duration().as_secs_f64();
        let r = (r.cos() + 0.5) / 2.0;

        frame.clear_color(r as f32, 0.0, 0.0, 0.0);
        self.mesh.draw(frame);
    }

    fn handle_event(&mut self, _frame_time : &support::FrameTime, event : glutin::Event) {
        use glutin::WindowEvent::*;
        use glutin::Event::WindowEvent;
        // use glutin::{ ControlFlow };

        match event {
            WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                CloseRequested => self.running = false,
                // Redraw the triangle when the window is resized.
                Resized(..) => {
                    // self.mesh.draw(frame);
                },
                _ => (),
            }
            _ => (),
        };
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



