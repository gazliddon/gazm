pub mod system;
pub mod frametime;
pub mod sampler;

use glium::{glutin};
use frametime::FrameTime;

pub trait App {
    fn draw(&self, display : &mut glium::Frame);

    fn handle_event(&mut self, _frame_time : &FrameTime, input_event : glutin::Event) {
        use glutin::WindowEvent::*;
        use glutin::Event::WindowEvent;
        if let WindowEvent {event, ..} = input_event {
            match event {
                ReceivedCharacter(ch) => self.handle_character(ch),
                CloseRequested => self.close_requested(),
                Resized(..) => {
                },
                _ => (),
            }
        };
    }

    fn ui(&self, _ui : &mut imgui::Ui) {
    }

    fn update(&mut self, frame_time : &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, c : char);
}


