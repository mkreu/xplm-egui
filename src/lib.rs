use egui::{ClippedMesh, output::OutputEvent};
use input::XplmInputState;
pub use misc_util::check_gl_error;
use painter::Painter;
use xplm::data::{borrowed::DataRef, ArrayRead};

mod input;
mod misc_util;
mod painter;

pub struct XplmGui {
    ctx: egui::CtxRef,
    painter: Painter,
    pub input_state: XplmInputState, //TODO proper abstraction
    clipped_meshes: Vec<ClippedMesh>,
    viewport: DataRef<[i32]>,
    has_keyboard_focus: bool,
}

impl XplmGui {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        Ok(Self {
            ctx: Default::default(),
            painter: Painter::new(gl)?,
            input_state: Default::default(),
            clipped_meshes: vec![],
            viewport: DataRef::find("sim/graphics/view/viewport").unwrap(),
            has_keyboard_focus: false,
        })
    }
    pub fn update(&mut self, window: &xplm::window::Window, run_ui: impl FnOnce(&egui::CtxRef)) {
        let input = self.gather_input();
        let (output, shapes) = self.ctx.run(input, run_ui);
        if !self.has_keyboard_focus && self.ctx.wants_keyboard_input() {
            window.take_keyboard_focus()
        } 
        if self.has_keyboard_focus && !self.ctx.wants_keyboard_input() {
            window.loose_keyboard_focus()
        } 
        handle_output(window, output);
        self.clipped_meshes = self.ctx.tessellate(shapes);
    }

    pub fn draw(&mut self, window: &xplm::window::Window, gl: &glow::Context) {
        self.painter.upload_egui_texture(gl, &self.ctx.font_image());
        let w_geo = window.geometry();
        let mut viter = self.viewport.as_vec().into_iter();
        let viewport = [
            viter.next().unwrap(),
            viter.next().unwrap(),
            viter.next().unwrap(),
            viter.next().unwrap(),
        ];
        self.painter
            .paint_meshes(gl, w_geo, viewport, 1.0, &self.clipped_meshes);
    }

}

fn handle_output(window: &xplm::window::Window, output: egui::Output) {
    for event in output.events {
        match event {
            OutputEvent::FocusGained(_) => {
                window.take_keyboard_focus()
            }
            _ => ()
        }
    }
}

impl XplmGui {
    fn gather_input(&mut self) -> egui::RawInput {
        self.input_state.take_egui_input()
    }
}
