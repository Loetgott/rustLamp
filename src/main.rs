mod color_picker;
mod artNetSender;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use crate::color_picker::ColorPicker;

fn main() {
    let app = Application::new(Some("com.loetgott.rustLamp"), Default::default());

    app.connect_activate(|app| {
        let window1 = ApplicationWindow::new(app);
        window1.set_title(Option::from("ColorPicker Test"));
        window1.set_decorated(true);
        window1.set_default_size(400, 300);

        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        window1.set_child(Some(&main_box));

        let color_picker = ColorPicker::new();
        main_box.append(color_picker.widget());

        let button = gtk4::Button::with_label("anwenden!");
        main_box.append(&button);
        window1.show();


    });

    app.run();
}
