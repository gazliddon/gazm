#[macro_use]
extern crate glium;
extern crate imgui;
extern crate cgmath;

mod support;
mod mesh;
mod frametime;

#[allow(unused_imports)]
use glium::{glutin, Surface};
#[allow(unused_imports)]
use glium::index::PrimitiveType;
use support::{ System };
use mesh::Mesh;

struct App {
    mesh : Box<dyn mesh::MeshTrait>,
    running : bool,
    frame_time : frametime::FrameTime
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
            frame_time : frametime::FrameTime::default()
        }
    }

}

pub fn cos01(x: f64) -> f64 { (x.cos() / 2.0) + 0.5 }
pub fn sin01(x: f64) -> f64 { (x.sin() / 2.0) + 0.5 }

impl support::App for App {

    fn draw(&self, frame : &mut glium::Frame) {
        use cgmath::*;

        let t = self.frame_time.now_as_duration().as_secs_f64();
        frame.clear_color(cos01(t * 10.0) as f32, 0.0, 0.0, 0.0);
        let m = Matrix4::<f32>::from_scale(cos01(t) as f32);
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

    fn update(&mut self, frame_time : &frametime::FrameTime) {
        self.frame_time = *frame_time;
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn ui(&self, ui : &mut imgui::Ui) {
        use imgui::*;

        // let mut open = true;

        // ui.show_demo_window(&mut open);

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
    let mut system = System::new();
    let mut app = App::new(&system);
    system.run_app(&mut app);
}



