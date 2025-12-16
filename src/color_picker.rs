use gtk4::prelude::*;
use gtk4::{self as gtk, Box as GtkBox, Stack};

pub struct ColorPicker {
    root: GtkBox,
}

impl ColorPicker {
    pub fn new() -> Self {
        let root = GtkBox::new(gtk::Orientation::Vertical, 8);
        let stack = Stack::new();

        let rgb_box = GtkBox::new(gtk::Orientation::Horizontal, 4);
        rgb_box.set_visible(true);
        let cmy_box = GtkBox::new(gtk::Orientation::Horizontal, 4);
        cmy_box.set_visible(true);
        let hsv_box = GtkBox::new(gtk::Orientation::Horizontal, 4);
        hsv_box.set_visible(true);
        stack.add_titled(&rgb_box, Some("rgb"), "RGB");
        stack.add_titled(&cmy_box, Some("cmy"), "CMY");
        stack.add_titled(&hsv_box, Some("hsv"), "HSV");
        stack.set_visible(true);

        let switcher = gtk::StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        root.append(&switcher);

        Self { root }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.root
    }
}
