pub mod frametime;
pub mod sampler;
pub mod system;

use frametime::FrameTime;
use glium::glutin;

pub trait App {
    fn draw(&self, display: &mut glium::Frame);

    fn handle_event(&mut self, _frame_time: &FrameTime, input_event: glutin::Event) -> bool {
        use glutin::Event::WindowEvent;
        use glutin::WindowEvent::*;
        let mut ret = false;

        if let WindowEvent { event, .. } = input_event {
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
        };
        ret
    }

    fn ui(&mut self, _ui: &mut imgui::Ui) {}

    fn update(&mut self, frame_time: &FrameTime);
    fn is_running(&self) -> bool;
    fn close_requested(&mut self);
    fn handle_character(&mut self, c: char);
    fn resize(&mut self, w : f64, h: f64);
}
