use imgui_glium_renderer::imgui;
use glium::glutin;

pub mod frametime;
pub mod sampler;
pub mod system;

use crate::v2::*;

use frametime::FrameTime;

pub trait App {
    fn draw(&self, hdpi : f64, pos : V2<isize>, dims: V2<usize>, display: &mut glium::Frame);


    fn handle_event(&mut self, input_event: &glutin::event::Event<()>) -> bool {
        use glutin::event::Event;
        let mut ret = false;

        match input_event {
            // Window events
            Event::WindowEvent {event, ..} => {
                use glutin::event::WindowEvent::*;
                match *event {
                    ReceivedCharacter(ch) => self.handle_character(ch),
                    CloseRequested => self.close_requested(),
                    Resized(l) => {
                        let w = l.width;
                        let h = l.height;
                        self.resize(w as f64,h as f64);
                        ret = true;
                    }
                    _ => (),
                }
            }

            // Device events
            Event::DeviceEvent { event, ..} => {
                use glutin::event::KeyboardInput;
                use glutin::event::DeviceEvent::*;
                use glutin::event::ElementState::*;

                println!("{:?}", event);

                match event {
                    Motion{..} => (),
                    Button{..} => (),
                    MouseMotion{..} => (),
                    Key(KeyboardInput {virtual_keycode, state, ..}) => {
                        if let Some(code) = virtual_keycode {
                            if *state == Pressed {
                                self.handle_key(code.clone())
                            }
                        }
                    },
                    _ => ()
                }
            }

            // Event::Awakened => self.awake(),
            Event::Suspended { .. } => self.suspend(),

            _ => (),
        };

        ret
    }

    fn awake(&mut self) {
    }

    fn suspend(&mut self) {
    }

    fn handle_key(&mut self, _code : glutin::event::VirtualKeyCode ) {
    }

    fn ui(&mut self, _hdpi : f64, _pos: V2<isize>,  _dims : V2<usize>, _ui: &mut imgui::Ui) {}

    fn update(&mut self, frame_time: &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, c: char);
    fn resize(&mut self, w : f64, h: f64);
}
