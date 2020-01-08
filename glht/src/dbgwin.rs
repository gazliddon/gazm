use super::simple::simplecore::Simple;

pub struct DbgWin {
    addr : u16
}

impl DbgWin {

    pub fn new(addr : u16) -> Self {
        Self {
            addr
        }
    }

    pub fn render(&self, ui: &imgui::Ui, machine : &Simple) {

        let [_,h] = ui.calc_text_size(im_str!( " " ), false, std::f32::MAX);
        let draw_list = ui.get_window_draw_list();

        let mut pos = ui.cursor_screen_pos();

        const WHITE: [f32; 3] = [1.0, 1.0, 1.0];
        const YELLOW: [f32; 3] = [1.0, 1.0, 0.0];

        let mut addr =  self.addr;

        let diss = machine.get_dissambler();

        draw_list.add_text(
            pos.clone(),
            YELLOW,
            "ADDR    CODE");

        pos[1] += h;

        for _i in 0..10 {

            let d = diss.diss(addr);

            let text = format!("{:04x}    {}", addr, d.text);

            draw_list.add_text(
                pos.clone(),
                WHITE,
                &text);

            pos[1] += h;

            addr = d.next_instruction_addr;
        }
    }

    pub fn next_instruction(&mut self, machine : &Simple) {
        let diss = machine.get_dissambler();
        let d = diss.diss(self.addr);
        self.addr = d.next_instruction_addr;
    }
}


