#[allow(unused_imports)]
#[macro_use]
extern crate imgui_winit_support;

#[allow(unused_imports)]
#[macro_use]
extern crate imgui_glium_renderer;

#[allow(unused_imports)]
#[macro_use]
pub extern crate glium;

#[macro_use]
extern crate log;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[allow(dead_code)]
mod styles;

#[allow(dead_code)]
mod app;
#[allow(dead_code)]
mod colour;
#[allow(dead_code)]
mod colourcell;
#[allow(dead_code)]
mod docwin;
#[allow(dead_code)]
mod events;
#[allow(dead_code)]
mod mesh;
#[allow(dead_code)]
mod scrbox;
#[allow(dead_code)]
mod simple;
#[allow(dead_code)]
mod sourcewin;
#[allow(dead_code)]
mod text;
#[allow(dead_code)]
mod textscreen;
#[allow(dead_code)]
mod v2;

pub use glium::glutin;
pub use imgui_glium_renderer::imgui;

use app::{frametime::FrameTime, system::System, App};

use glium::index::PrimitiveType;
use glium::Surface;
use imgui::{im_str, Condition, Ui, Window};
use mesh::Mesh;
use v2::*;

use simple::{ SimpleMachine, SimpleMem } ;

#[allow(dead_code)]
struct MyApp {
    mesh: Box<dyn mesh::MeshTrait>,
    running: bool,
    frame_time: FrameTime,
    machine: SimpleMachine<SimpleMem>,
    // dbgwin: dbgwin::DbgWin,
    sourcewin: sourcewin::SourceWin,
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
        let machine = simple::make_simple(sym_file);

        let mesh = make_mesh(&system);

        Self {
            machine,
            mesh,
            running: true,
            frame_time: FrameTime::default(),
            // dbgwin: dbgwin::DbgWin::new(0x9900),
            sourcewin: sourcewin::SourceWin::new(),
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

impl KeyPress {}

trait ToArray<U> {
    fn as_array(&self) -> [U; 2];
}

impl<U> ToArray<U> for V2<U>
where
    U: Copy + Clone,
{
    fn as_array(&self) -> [U; 2] {
        [self.x, self.y]
    }
}

impl App<events::Events> for MyApp {
    fn draw(&self, _hdpi: f64, _pos: V2<isize>, _dims: V2<usize>, frame: &mut glium::Frame) {
        use cgmath::*;

        let t = self.frame_time.now_as_duration().as_secs_f64();
        frame.clear_color(cos01(t * 10.0) as f32, 0.0, 0.0, 0.0);
        let m = Matrix4::<f32>::from_scale(cos01(t) as f32);
        self.mesh.draw(m, frame);
    }

    fn handle_key(
        &mut self,
        _code: glutin::event::VirtualKeyCode,
        mstate: glutin::event::ModifiersState,
    ) ->Option<events::Events> {
        use events::Events::*;
        use glutin::event::VirtualKeyCode as Vk;
        use simple::Machine;

        let target = &mut self.sourcewin;

        let ret_event = if mstate.ctrl() {
            match _code {
                Vk::J => target.event(ScrollUp),
                Vk::K => target.event(ScrollDown),
                _ => None,
            }
        } else if mstate.is_empty() {
            match _code {
                Vk::Q => { self.close_requested();None },
                Vk::J => target.event(CursorDown),
                Vk::K => target.event(CursorUp),
                Vk::Space => target.event(Space),
                Vk::S => { self.machine.step();None },
                _ => None,
            }
        } else {
            None
        };

        ret_event
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

    fn resize(&mut self, w: f64, h: f64) {
        let dims = V2::new(w, h);
        self.sourcewin.resize(dims.as_usizes());
    }

    fn ui(&mut self, hdpi: f64, _pos: V2<isize>, dims: V2<usize>, ui: &mut Ui) {
        use text::Dimensions;

        let char_dims = ui.current_font().dims() / hdpi as f32;
        let grid_cell_dims = &dims.as_f32s().div_components(char_dims).as_usizes();

        // println!("dims: {:?} gcd: {:?} cd ; {:?}", dims, char_dims, grid_cell_dims);
        
        use simple::Machine;

        let machine = &self.machine;
        let pc = machine.get_regs().pc;
        let sources = &machine.get_rom().sources;
        let state = machine.get_state();

        if self.sourcewin.is_empty() {
            if let Some(sf) = sources.addr_to_loc(pc).map(|l| sources.get(&l.file)).flatten() {
                self.sourcewin.set_source_file(sf.clone());
            }
        }

        self.sourcewin.update(grid_cell_dims, &self.frame_time, pc, state );

        let pos = V2::new(0.0, 0.0);

        Window::new(im_str!("Hello world"))
            .bg_alpha(0.9)
            .size(dims.as_f32s().as_array(), Condition::Always)
            .no_decoration()
            .position([0.0, 0.0], Condition::Always)
            .movable(false)
            .build(ui, || {
                let tc = text::ImgUiTextRender::new(&pos, &char_dims, grid_cell_dims, &ui);
                self.sourcewin.render(&tc);
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
