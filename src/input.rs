use egui::{Modifiers, Pos2, Rect};

#[derive(Default)]
pub struct XplmInputState {
    //start_time: instant::Instant,
    egui_input: egui::RawInput,
    //pointer_pos_in_points: Option<egui::Pos2>,
    //any_pointer_button_down: bool,
    //current_cursor_icon: egui::CursorIcon,
    //current_pixels_per_point: f32,

    //clipboard: clipboard::Clipboard,
    //screen_reader: screen_reader::ScreenReader,
}

impl XplmInputState {
    pub fn take_egui_input(&mut self) -> egui::RawInput {
        self.egui_input.take()
    }

    pub fn keyboard_event(
        &mut self,
        _window: &xplm::window::Window,
        event: xplm::window::KeyEvent,
    ) {
        // TODO char input
        self.egui_input.modifiers = Modifiers {
            alt: event.option_pressed(),
            ctrl: event.control_pressed(),
            shift: event.shift_pressed(),
            mac_cmd: false,
            command: event.control_pressed(),
        };
        let pressed = match event.action() {
            xplm::window::KeyAction::Press => true,
            xplm::window::KeyAction::Release => false,
        };
        if pressed {
            if let Some(c) = event.char() {
                self.egui_input
                    .events
                    .push(egui::Event::Text(c.to_string()))
            }
        }

        if let Some(key) = xplm_to_egui_key(&event) {
            self.egui_input.events.push(egui::Event::Key {
                key,
                pressed,
                modifiers: self.egui_input.modifiers,
            })
        }
    }

    pub fn mouse_event(
        &mut self,
        window: &xplm::window::Window,
        event: xplm::window::MouseEvent,
    ) -> bool {
        let geo = window.geometry();
        let pos = Pos2::new(
            (event.position().x() - geo.left()) as f32,
            (geo.top() - event.position().y()) as f32,
        );
        //let pos = Pos2::new(event.position().x() as f32, event.position().y() as f32);

        let event = match event.action() {
            xplm::window::MouseAction::Drag => egui::Event::PointerMoved(pos),
            xplm::window::MouseAction::Down => egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: self.egui_input.modifiers,
            },
            xplm::window::MouseAction::Up => egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: self.egui_input.modifiers,
            },
        };
        self.egui_input.events.push(event);
        false
    }

    pub fn scroll_event(
        &mut self,
        _window: &xplm::window::Window,
        _event: xplm::window::ScrollEvent,
    ) -> bool {
        false
    }

    pub fn cursor(
        &mut self,
        window: &xplm::window::Window,
        position: xplm::geometry::Point<i32>,
    ) -> xplm::window::Cursor {
        let geo = window.geometry();
        self.egui_input.screen_rect = Some(Rect {
            min: Pos2 { x: 0.0, y: 0.0 },
            max: Pos2 {
                x: (geo.right() - geo.left()) as f32,
                y: (geo.top() - geo.bottom()) as f32,
            },
        });

        //let pos = Pos2::new(position.x() as f32, position.y() as f32);
        let pos = Pos2::new(
            (position.x() - geo.left()) as f32,
            (geo.top() - position.y()) as f32,
        );
        self.egui_input.events.push(egui::Event::PointerMoved(pos));
        xplm::window::Cursor::Default
    }
}
fn xplm_to_egui_key(xplm_event: &xplm::window::KeyEvent) -> Option<egui::Key> {
    use egui::Key as e;
    use xplm::window::Key as x;
    let key = match xplm_event.key() {
        x::Left => e::ArrowLeft,
        x::Up => e::ArrowUp,
        x::Right => e::ArrowRight,
        x::Down => e::ArrowDown,

        x::Back => e::Backspace,
        x::Tab => e::Tab,
        x::Return => e::Enter,
        x::Enter => e::Enter,
        x::Escape => e::Escape,
        x::Space => e::Space,

        x::End => e::End,
        x::Home => e::Home,
        x::Insert => e::Insert,
        x::Delete => e::Delete,

        x::Key0 | x::Numpad0 => e::Num0,
        x::Key1 | x::Numpad1 => e::Num1,
        x::Key2 | x::Numpad2 => e::Num2,
        x::Key3 | x::Numpad3 => e::Num3,
        x::Key4 | x::Numpad4 => e::Num4,
        x::Key5 | x::Numpad5 => e::Num5,
        x::Key6 | x::Numpad6 => e::Num6,
        x::Key7 | x::Numpad7 => e::Num7,
        x::Key8 | x::Numpad8 => e::Num8,
        x::Key9 | x::Numpad9 => e::Num9,

        x::A => e::A,
        x::B => e::B,
        x::C => e::C,
        x::D => e::D,
        x::E => e::E,
        x::F => e::F,
        x::G => e::G,
        x::H => e::H,
        x::I => e::I,
        x::J => e::J,
        x::K => e::K,
        x::L => e::L,
        x::M => e::M,
        x::N => e::N,
        x::O => e::O,
        x::P => e::P,
        x::Q => e::Q,
        x::R => e::R,
        x::S => e::S,
        x::T => e::T,
        x::U => e::U,
        x::V => e::V,
        x::W => e::W,
        x::X => e::X,
        x::Y => e::Y,
        x::Z => e::Z,
        _ => {
            return None;
        }
    };
    Some(key)
}
