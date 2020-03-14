pub mod frametime;
pub mod sampler;
pub mod system;

use frametime::FrameTime;
use glium::glutin;

pub trait App {
    fn draw(&self, display: &mut glium::Frame);

    fn handle_event(&mut self, _frame_time: &FrameTime, input_event: glutin::Event) -> bool {
        use glutin::Event;
        let mut ret = false;

        match input_event {
            // Window events
            Event::WindowEvent {event, ..} => {
                use glutin::WindowEvent::*;
                match event {
                    ReceivedCharacter(ch) => self.handle_character(ch),
                    CloseRequested => self.close_requested(),
                    Resized(l) => {
                        let w = l.width;
                        let h = l.height;
                        self.resize(w,h);
                        ret = true;
                    }
                    _ => (),
                }
            }

            // Device events
            Event::DeviceEvent { event, ..} => {
                use glutin::KeyboardInput;
                use glutin::DeviceEvent::*;
                use glutin::ElementState::*;

                match event {
                    Motion{..} => (),
                    Button{..} => (),
                    MouseMotion{..} => (),
                    Key(KeyboardInput {virtual_keycode, modifiers, state, ..}) => {
                        if let Some(code) = virtual_keycode {
                            if state == Pressed {
                                self.handle_key(code,modifiers)
                            }
                        }
                    },
                    _ => ()
                }
            }

            Event::Awakened => self.awake(),
            Event::Suspended { .. } => self.suspend(),
        };

        ret
    }

    fn awake(&mut self) {
    }

    fn suspend(&mut self) {
    }

    fn handle_key(&mut self, _code : glutin::VirtualKeyCode, _mods : glutin::ModifiersState) {
    }

    fn ui(&mut self, _ui: &mut imgui::Ui) {}
    fn update(&mut self, frame_time: &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, c: char);
    fn resize(&mut self, w : f64, h: f64);
}
