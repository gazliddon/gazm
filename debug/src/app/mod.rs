use glium::glutin::event;
use event::{ ModifiersState, DeviceEvent };
use imgui_glium_renderer::imgui;
use imgui_glium_renderer::glium;

pub mod frametime;
pub mod sampler;
pub mod system;

use crate::v2::*;

use frametime::FrameTime;

pub trait App<EV> {
    fn draw(&self, hdpi: f64, pos: V2<isize>, dims: V2<usize>, display: &mut glium::Frame);

    fn handle_event(
        &mut self,
        input_event: &event::Event<()>,
        mstate: ModifiersState,
    ) -> bool {
        use event::{ElementState, Event, KeyboardInput, WindowEvent};
        let mut ret = false;

        match input_event {
            // Window events
            Event::WindowEvent { event, .. } => match *event {
                WindowEvent::KeyboardInput { input, .. } => {
                    let KeyboardInput {
                        state,
                        virtual_keycode,
                        ..
                    } = input;

                    if let Some(v_code) = virtual_keycode {
                        if state == ElementState::Pressed {
                            self.handle_key(v_code, mstate);
                        }
                    }
                }

                WindowEvent::ReceivedCharacter(ch) => self.handle_character(ch, mstate),

                WindowEvent::CloseRequested => self.close_requested(),
                WindowEvent::Resized(l) => {
                    let w = l.width;
                    let h = l.height;
                    self.resize(w as f64, h as f64);
                    ret = true;
                }
                _ => (),
            },

            // Device events
            Event::DeviceEvent { event, .. } => {

                match event {
                    DeviceEvent::Motion { .. } => (),
                    DeviceEvent::Button { .. } => (),
                    DeviceEvent::MouseMotion { .. } => (),
                    _ => (),
                }
            }

            // Event::Awakened => self.awake(),
            Event::Suspended { .. } => self.suspend(),

            _ => (),
        };

        ret
    }

    fn awake(&mut self) {}

    fn suspend(&mut self) {}

    fn handle_key(
        &mut self,
        _code: event::VirtualKeyCode,
        _mstate: event::ModifiersState,
    ) -> Option<EV> {
        None
    }

    fn ui(&mut self, _hdpi: f64, _pos: V2<isize>, _dims: V2<usize>, _ui: &mut imgui::Ui) {}

    fn update(&mut self, frame_time: &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, _c: char, _mstate: event::ModifiersState) {
    }

    fn resize(&mut self, w: f64, h: f64);
}
