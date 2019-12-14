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
    fn draw(&self, display : &glium::Display) {
        self.mesh.draw(&display);
    }

    fn handle_event(&mut self, _dt : f64, display : &glium::Display, event : glutin::Event) {
        use glutin::WindowEvent::*;
        use glutin::Event::WindowEvent;
        // use glutin::{ ControlFlow };

        match event {
            WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                CloseRequested => self.running = false,
                // Redraw the triangle when the window is resized.
                Resized(..) => {
                    self.mesh.draw(display);
                },
                _ => (),
            }
            _ => (),
        };
    }

    fn update(&mut self, _dt : f64) {
    }

    fn is_running(&self) -> bool {
        true
    }
}


fn main() {
    let mut system = System::new();

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

    // Draw the triangle to the screen.
    mesh.draw(&system.display);

    let run = |_display : &glium::Display, event | -> glutin::ControlFlow {
        use glutin::WindowEvent::*;
        use glutin::Event::WindowEvent;
        use glutin::{ ControlFlow };

        match event {
            WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                CloseRequested => ControlFlow::Break,
                // Redraw the triangle when the window is resized.
                Resized(..) => {
                    // mesh.draw(display);
                    ControlFlow::Continue
                },
                _ => ControlFlow::Continue,
            }
            _ => ControlFlow::Continue,
        }
    };

    let draw = |display : &glium::Display| {
        mesh.draw(display);
    };

    loop {
        system.run(run);
        system.display(draw);
    };
}


