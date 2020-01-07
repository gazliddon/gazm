pub mod frametime;
pub mod sampler;
pub mod system;

use frametime::FrameTime;
use glium::glutin;

pub trait App {
    fn draw(&self, display: &mut glium::Frame);

    fn handle_event(&mut self, _frame_time: &FrameTime, input_event: glutin::Event) {
        use glutin::Event::WindowEvent;
        use glutin::WindowEvent::*;
        if let WindowEvent { event, .. } = input_event {
            match event {
                ReceivedCharacter(ch) => self.handle_character(ch),
                CloseRequested => self.close_requested(),
                Resized(..) => {}
                _ => (),
            }
        };
    }

    fn ui(&mut self, _ui: &mut imgui::Ui) {}

    fn update(&mut self, frame_time: &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, c: char);
}
