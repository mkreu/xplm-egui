use std::sync::mpsc::Receiver;

use egui::{ClippedMesh, Pos2, RawInput, Rect};
use painter::Painter;
use xplm::{data::{borrowed::DataRef, ArrayRead}, debugln};
pub use misc_util::check_gl_error;

mod input;
mod misc_util;
mod painter;

pub struct XplmGui {
    ctx: egui::CtxRef,
    painter: Painter,
    gathered_input: RawInput,
    clipped_meshes: Vec<ClippedMesh>,
    viewport: DataRef<[i32]>,
}

impl XplmGui {
    pub fn new(gl: &glow::Context) -> Result<Self, String> {
        let mut ctx = Default::default();
        Ok(Self {
            ctx,
            painter: Painter::new(gl)?,
            gathered_input: Default::default(),
            clipped_meshes: vec![],
            viewport: DataRef::find("sim/graphics/view/viewport").unwrap(),
        })
    }
    pub fn update(&mut self, run_ui: impl FnOnce(&egui::CtxRef)) {
        let input = self.gather_input();
        let (output, shapes) = self.ctx.run(input, run_ui);
        self.clipped_meshes = self.ctx.tessellate(shapes);
    }

    pub fn draw(&mut self, window: &xplm::window::Window, gl: &glow::Context) {
        self.painter.upload_egui_texture(gl, &self.ctx.font_image());
        let w_geo = window.geometry();
        //self.gathered_input.append(RawInput {
        //    screen_rect: Some(Rect::from_min_max(
        //        Pos2::new(w_geo.left() as f32, w_geo.bottom() as f32),
        //        Pos2::new(w_geo.right() as f32, w_geo.top() as f32),
        //    )),
        //    ..Default::default()
        //});
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

    fn keyboard_event(&mut self, _window: &xplm::window::Window, _event: xplm::window::KeyEvent) {}

    fn mouse_event(
        &mut self,
        _window: &xplm::window::Window,
        _event: xplm::window::MouseEvent,
    ) -> bool {
        true
    }

    fn scroll_event(
        &mut self,
        _window: &xplm::window::Window,
        _event: xplm::window::ScrollEvent,
    ) -> bool {
        true
    }

    fn cursor(
        &mut self,
        _window: &xplm::window::Window,
        _position: xplm::geometry::Point<i32>,
    ) -> xplm::window::Cursor {
        xplm::window::Cursor::Default
    }
}
impl XplmGui {
    fn gather_input(&mut self) -> egui::RawInput {
        self.gathered_input.take()
    }
}
