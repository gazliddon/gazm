#[allow(unused_imports)]
#[macro_use] extern crate imgui_winit_support;

#[allow(unused_imports)]
#[macro_use] extern crate imgui_glium_renderer;

#[allow(unused_imports)]
#[macro_use] pub extern crate glium;

#[macro_use] extern crate log;

#[allow(unused_imports)]
#[macro_use] extern crate serde_derive;
#[allow(dead_code)] mod styles;

#[allow(dead_code)] mod app;
#[allow(dead_code)] mod sourcewin;
#[allow(dead_code)] mod mesh;
#[allow(dead_code)] mod simple;
#[allow(dead_code)] mod dbgwin;
#[allow(dead_code)] mod textscreen;
#[allow(dead_code)] mod events;
#[allow(dead_code)] mod docwin;
#[allow(dead_code)] mod colour;
#[allow(dead_code)] mod colourcell;
#[allow(dead_code)] mod scrbox;
#[allow(dead_code)] mod text;
#[allow(dead_code)] mod v2;

pub use imgui_glium_renderer::imgui;
pub use glium::glutin;

use app::{frametime::FrameTime, system::System, App};

use glium::index::PrimitiveType;
use glium::Surface;
use mesh::Mesh;
use v2::*;

#[allow(dead_code)]
struct MyApp {
    mesh: Box<dyn mesh::MeshTrait>,
    running: bool,
    frame_time: FrameTime,
    machine : Box<dyn simple::simplecore::Machine>,
    dbgwin : dbgwin::DbgWin,
    sourcewin : sourcewin::SourceWin,
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

        let sym_file = "./asm/out/demo.syms";
        let machine = Box::new(simple::simplecore::make_simple(sym_file));

        let mesh = make_mesh(&system);

        Self {
            machine,
            mesh,
            running: true,
            frame_time: FrameTime::default(),
            dbgwin: dbgwin::DbgWin::new(0x9900),
            sourcewin : sourcewin::SourceWin::new()
        }
    }
}

pub fn cos01(x: f64) -> f64 {
    (x.cos() / 2.0) + 0.5
}
pub fn sin01(x: f64) -> f64 {
    (x.sin() / 2.0) + 0.5
}

use glutin::event::VirtualKeyCode;

#[allow(dead_code)]
enum KeyPress {
    Unknown(VirtualKeyCode),
    Bare(VirtualKeyCode),
    Ctrl(VirtualKeyCode),
    Alt(VirtualKeyCode),
    Shift(VirtualKeyCode),
    CtrlAlt(VirtualKeyCode),
    CtrlShift(VirtualKeyCode),
    CtrlAltShift(VirtualKeyCode),
}

impl KeyPress {
    pub fn new(key : VirtualKeyCode) -> Self {
        // let key = key.clone();
             Self::Bare(key)
    }
}




trait ToArray<U> {
    fn as_array(&self) -> [U;2];
}

impl<U> ToArray<U> for V2<U> 
where U : Copy + Clone
{
    fn as_array(&self) -> [U;2] {
        [self.x, self.y]
    }
}


impl App for MyApp {
    fn draw(&self, _dims: V2<usize>,frame: &mut glium::Frame) {
        use cgmath::*;

        let t = self.frame_time.now_as_duration().as_secs_f64();
        frame.clear_color(cos01(t * 10.0) as f32, 0.0, 0.0, 0.0);
        let m = Matrix4::<f32>::from_scale(cos01(t) as f32);
        self.mesh.draw(m, frame);
    }

    fn handle_key(&mut self, code : glutin::event::VirtualKeyCode) {
        use glutin::event::VirtualKeyCode as VK;

        let dbgwin = &mut self.sourcewin;

        let kp = KeyPress::new(code);

        use KeyPress::*;
        use events::Events::*;

        match kp {
            Bare(VK::Q) => {
                self.close_requested();
            },

            Bare(VK::J) => {
                dbgwin.event(CursorDown);
            }

            Bare(VK::K) => {
                dbgwin.event(CursorUp);
            }

            Ctrl(VK::D) => {
                dbgwin.event(PageDown);
            }

            Ctrl(VK::U) => {
                dbgwin.event(PageUp);
            }

            _ => (),
        }
    }

    fn handle_character(&mut self, c: char) {

        if c == 'q' {
            self.close_requested()
        }

        use events::Events::*;
        let dbgwin = &mut self.sourcewin;

        if c == 'i' {
            self.dbgwin.event(ScrollUp);
        }

        if c == 'o'  {
            dbgwin.event(ScrollDown);
        }

        if c == 'j' {
            dbgwin.event(CursorDown);
        }

        if c == 'k' {
            dbgwin.event(CursorUp);
        }

        if c == ' ' {
            dbgwin.event(Space);
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

    fn resize(&mut self, w : f64, h: f64) {
        let dims = V2{x : w as usize, y : h as usize};
        self.sourcewin.resize(dims);
    }

    fn ui(&mut self, dims : V2<usize>, ui: &mut imgui::Ui) {
        use imgui::*;

        let size = dims.as_f32s().as_array();

        let machine = self.machine.as_ref();
        let pc = machine.get_regs().pc;
        let sources = &machine.get_rom().sources;

        let win_dims = text::TextWinDims::new(&*ui);

        self.sourcewin.update(win_dims, &self.frame_time,sources, pc);

        Window::new(im_str!("Hello world"))
            .bg_alpha(1.0)
            .size(size, Condition::Always)
            .no_decoration()
            .position([0.0, 0.0], Condition::Always)
            .movable(false)
            .build(ui, || {
                let tc = text::ImgUiTextRender::new(&ui);
                self.sourcewin.render(&tc, sources);
            });
    }
}

fn main() {
    use std::env;

    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let system = System::new();
    let app = MyApp::new(&system);

    system.main_loop(app);
}
