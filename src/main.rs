mod color_picker;
mod art_net_sender;
mod color;

use std::cell::RefCell;
use std::rc::Rc;
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

        let mut color = color::Color::new();


        let color = Rc::new(RefCell::new(color::Color::new()));
        let color_picker = ColorPicker::new(color.clone());
        main_box.append(color_picker.widget());

        let color_clone = color.clone();
        let button = gtk4::Button::with_label("Farbe ausgeben");
        button.connect_clicked(move |_| {
            let c = color_clone.borrow();
            println!(
                "Gewählte Farbe - R: {}, G: {}, B: {}",
                c.red, c.green, c.blue
            );
        });

        main_box.append(&button);
        window1.show();


    });

    app.run();
}
