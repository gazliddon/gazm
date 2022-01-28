#![allow(unused_imports)]
#![allow(dead_code)]

mod app;
mod dap;
mod colour;
mod colourcell;
mod cycler;
mod docwin;
mod events;
mod mesh;
mod scrbox;
mod simple;
mod sourcewin;
mod styles;
mod text;
mod textscreen;
mod v2;

use app::{frametime::FrameTime, system::System, App};
use byteorder::{BigEndian, ByteOrder};
use glium::index::PrimitiveType;
use glium::Surface;
use glium::{glutin, implement_vertex};
use imgui::{im_str, Condition, Ui, Window};
use imgui_glium_renderer::imgui;
use log::info;
use mesh::Mesh;
use romloader::sources::SourceDatabase;
use v2::*;

use simple::{Machine, SimpleMachine, SimpleMem};
use emu::breakpoints::{BreakPoint, BreakPointTypes, BreakPoints};

struct MyApp {
    mesh: Box<dyn mesh::MeshTrait>,
    running: bool,
    frame_time: FrameTime,
    machine: SimpleMachine<SimpleMem<BigEndian>>,
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

    Box::new(Mesh::new(system, vertex_buffer, index_buffer))
}

fn load_binary(filename : &str) -> Vec<u8> {
    use std::fs::File;
    use std::io::Read;
    let mut f = File::open(&filename).expect("no file found");
    let metadata = std::fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    buffer
}


impl MyApp {
    fn toggle_breakpoint_at_cursor(&mut self, bp_type: BreakPointTypes) {
        use emu::breakpoints::BreakPointTypes::*;
        self.break_point_fn_mut(|addr, break_points| {
            if let Some(bp) = break_points.find_breakpoint_id(addr, bp_type) {
                break_points.remove_by_id(bp);
            } else {
                break_points.add(addr, bp_type);
            }
        });
    }

    fn get_source_line(&self) {
        panic!()
        // use romloader::sources::SourceDatabaseTrait;
        // self.sourcewin.get_cursor_file_loc().and_then(|loc| {
        //     self.machine
        //         .get_rom()
        //         .sources
        //         .loc_to_source_line(&loc)
        // })
    }

    fn break_point_fn_mut(&mut self, _f: impl Fn(u16, &mut BreakPoints)) {
        panic!()
        // if let Some(addr) = self.get_source_line().and_then(|sl| sl.addr) {
        //     let bp = self.machine.get_breakpoints_mut();
        //     f(addr, bp);
        // };
    }

    fn break_points_at_addr_fn_mut(&mut self, f: impl Fn(Vec<&mut BreakPoint>)) {
        self.break_point_fn_mut(|addr, breakpoints| {
            let bp = breakpoints.get_breakpoints_mut(addr, 1);
            f(bp);
        });
    }

    pub fn toggle_breakpoints_at_addr(&mut self) {
        self.break_points_at_addr_fn_mut(|mut breakpoints| {
            for i in breakpoints.iter_mut() {
                i.toggle_active();
            }
        });
    }

    pub fn new(system: &System) -> Self {
        use emu::mem::MemoryIO;
        use romloader::sources::SourceDatabase;
        use emu::breakpoints::{BreakPoint, BreakPointTypes};

        let bin_file = "./out/a.bin";
        let sym_file = "./out/a.syms.json";

        let sd = SourceDatabase::from_json(sym_file);

        let bin_data = load_binary(bin_file);
        let slice = &bin_data[0x9900..];
        let mut mem = SimpleMem::default();
        mem.upload(0x9900, slice).expect("couldn't upload");
        let mut machine = SimpleMachine::new(mem,sd);
        machine.reset().unwrap();

        // FIX : Remove!
        let bps = machine.get_breakpoints_mut();
        bps.add(0x9904, BreakPointTypes::EXEC);

        let mesh = make_mesh(system);

        Self {
            machine,
            mesh,
            running: true,
            frame_time: FrameTime::default(),
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
    ) -> Option<events::Events> {
        use events::Events::*;
        use glutin::event::VirtualKeyCode as Vk;
        use simple::Machine;

        let target = &mut self.sourcewin;

        if mstate.ctrl() {
            match _code {
                Vk::J => target.event(&self.machine, ScrollUp),
                Vk::K => target.event(&self.machine, ScrollDown),
                Vk::D => target.event(&self.machine, PageUp),
                Vk::U => target.event(&self.machine, PageDown),
                _ => None,
            }
        } else if mstate.is_empty() {
            match _code {
                Vk::R => {
                    self.machine.set_state(simple::SimState::Running);
                    None
                },

                Vk::Q => {
                    self.close_requested();
                    None
                }

                Vk::J => target.event(&self.machine,CursorDown),
                Vk::K => target.event(&self.machine,CursorUp),
                Vk::Space => target.event(&self.machine,Space),
                Vk::S => {
                    self.machine.step().expect("Handle this");
                    target.event(&self.machine,Step);
                    None
                }
                Vk::B => {
                    self.toggle_breakpoint_at_cursor(BreakPointTypes::EXEC);
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn close_requested(&mut self) {
        self.running = false;
    }

    fn update(&mut self, frame_time: &FrameTime) {
        self.frame_time = *frame_time;
        let _ = self.machine.update();
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn resize(&mut self, w: f64, h: f64) {
        let dims = V2::new(w, h);
        self.sourcewin.resize(dims.as_usizes());
    }

    fn ui(&mut self, hdpi: f64, _pos: V2<isize>, dims: V2<usize>, ui: &mut Ui) {
        // use romloader::sources::SourceDatabaseTrait;
        use text::Dimensions;

        let char_dims = ui.current_font().dims() / hdpi as f32;
        let grid_cell_dims = &dims.as_f32s().div_components(char_dims).as_usizes();

        use simple::Machine;

        let machine = &self.machine;
        // let pc = machine.get_regs().pc;


        // let sources = &machine.get_rom().sources;

        // if self.sourcewin.is_empty() {
        //     if let Some(sf) = sources
        //         .addr_to_source_line(pc)
        //         .and_then(|l| sources.get(&l.file))
        //     {
        //         self.sourcewin.set_source_file(sf.clone());
        //     }
        // }

        self.sourcewin
            .update(grid_cell_dims, &self.frame_time, machine);

        let pos = V2::new(0.0, 0.0);

        Window::new(im_str!("Hello world"))
            .bg_alpha(0.9)
            .size(dims.as_f32s().as_array(), Condition::Always)
            .no_decoration()
            .position([0.0, 0.0], Condition::Always)
            .movable(false)
            .build(ui, || {
                let tc = text::ImgUiTextRender::new(&pos, &char_dims, grid_cell_dims, ui);
                self.sourcewin.render(&tc, machine);
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
