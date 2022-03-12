use anyhow::{anyhow, Result};
use xplm::{
    debugln,
    geometry::Rect,
    menu::{ActionItem, Menu, MenuClickHandler},
    plugin::{Plugin, PluginInfo},
    window::{Window, WindowDelegate, WindowRef},
    xplane_plugin,
};
use xplm_egui::XplmGuiContext;

xplane_plugin!(MinimalPlugin);

struct MinimalPlugin {
    _menu: Menu,
}

impl Plugin for MinimalPlugin {
    type Error = anyhow::Error;

    fn start() -> Result<Self> {
        // Build a GUI context that allows us to render the UI later on
        let gui = XplmGuiContext::new().map_err(|e_str| anyhow!(e_str))?;

        // Create the window that our gui should draw in
        let window = Window::new(
            Rect::from_left_top_right_bottom(0, 0, 800, 600),
            MyWindowDelegate {
                gui,
                name: "John".to_string(),
                age: 42,
            },
            xplm::window::WindowOptions::default(),
        );

        // Register a menu to open our window
        let menu = Menu::new("Hello Egui")?;
        menu.add_child(ActionItem::new("Show Window", ShowWindowHandler(window))?);
        menu.add_to_plugins_menu();
        Ok(MinimalPlugin { _menu: menu })
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("Egui-Hello"),
            signature: String::from("mkreu.egui-xplm.hello"),
            description: String::from("A hello world plugin rendering an egui ui inside X-Plane"),
        }
    }
}

struct MyWindowDelegate {
    gui: XplmGuiContext,
    name: String,
    age: u32,
}

impl WindowDelegate for MyWindowDelegate {
    fn draw(&mut self, window: &Window) {
        self.gui.update(window, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("My egui Application");
                ui.horizontal(|ui| {
                    ui.label("Your name: ");
                    ui.text_edit_singleline(&mut self.name);
                });
                ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                if ui.button("Click each year").clicked() {
                    self.age += 1;
                }
                ui.label(format!("Hello '{}', age {}", self.name, self.age));
            });
        });
        self.gui.draw(window);
    }

    fn keyboard_event(&mut self, window: &Window, event: xplm::window::KeyEvent) {
        self.gui.input_state.keyboard_event(window, event);
    }

    fn mouse_event(&mut self, window: &Window, event: xplm::window::MouseEvent) -> bool {
        self.gui.input_state.mouse_event(window, event)
    }

    fn scroll_event(&mut self, window: &Window, event: xplm::window::ScrollEvent) -> bool {
        self.gui.input_state.scroll_event(window, event)
    }

    fn cursor(
        &mut self,
        window: &Window,
        position: xplm::geometry::Point<i32>,
    ) -> xplm::window::Cursor {
        self.gui.input_state.cursor(window, position)
    }
}

struct ShowWindowHandler(WindowRef);

impl MenuClickHandler for ShowWindowHandler {
    fn item_clicked(&mut self, _item: &xplm::menu::ActionItem) {
        let (x, y) = xplm::window::get_mouse_location_global();
        debugln!("[Rust Plugin] mouse location: {x},{y}");
        self.0
            .set_geometry(Rect::from_left_top_right_bottom(x, y, x + 800, y - 600));
        self.0.set_visible(true);
        debugln!("{:?}", self.0.geometry());
    }
}
