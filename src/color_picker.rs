use gtk4::prelude::*;
use gtk4::{self as gtk, cairo, Box as GtkBox, DrawingArea, Stack};
use std::cell::RefCell;
use std::rc::Rc;
use crate::color::Color;

pub struct ColorPicker {
    root: GtkBox,
    color: Rc<RefCell<Color>>,
}

impl ColorPicker {
    pub fn new(color: Rc<RefCell<Color>>) -> Self {

        // Winkel der Maus & Linksklickstatus
        let mouse_angle = Rc::new(RefCell::new(0.0));
        let left_button_pressed = Rc::new(RefCell::new(false));

        let root = GtkBox::new(gtk::Orientation::Vertical, 8);
        let stack = Stack::new();

        let rgb_box = GtkBox::new(gtk::Orientation::Horizontal, 4);
        rgb_box.set_visible(true);
        let cmy_box = GtkBox::new(gtk::Orientation::Horizontal, 4);
        cmy_box.set_visible(true);
        let hsv_box = GtkBox::new(gtk::Orientation::Horizontal, 4);

        // Zeichnung
        let drawing = DrawingArea::new();
        drawing.set_content_width(500);
        drawing.set_content_height(500);

        // Draw-Funktion
        {
            let mouse_angle = mouse_angle.clone();
            let color_for_draw = color.clone();
            drawing.set_draw_func(move |_, cr, width, height| {
                let cx = width as f64 / 2.0;
                let cy = height as f64 / 2.0;
                let circle_width = 10.0;
                let radius = cx.min(cy) - circle_width;

                // HSV-Ring
                let offset = -std::f64::consts::PI / 2.0; // 0° (Rot) nach oben
                for i in 0..360 {
                    let angle1 = (i as f64).to_radians() + offset - 0.005;
                    let angle2 = ((i + 1) as f64).to_radians() + offset;
                    let (r, g, b) = hsv_to_rgb(i as f64 / 360.0, 1.0, 1.0);

                    cr.set_source_rgb(r, g, b);
                    cr.set_line_width(circle_width * 2.0);
                    cr.arc(cx, cy, radius, angle1, angle2);
                    cr.stroke().unwrap();
                }

                // Dreieck in der Mitte
                let triangle_radius = radius - circle_width;
                let triangle_angle = 2.0 * std::f64::consts::PI / 3.0;
                let angle_offset = *mouse_angle.borrow();

                let mut points = Vec::new();
                for j in 0..3 {
                    let angle = -std::f64::consts::PI / 2.0 + j as f64 * triangle_angle + angle_offset;
                    let x = cx + triangle_radius * angle.cos();
                    let y = cy + triangle_radius * angle.sin();
                    points.push((x, y));
                }

                let angle = -*mouse_angle.borrow();
                let hue = ((-angle).rem_euclid(2.0 * std::f64::consts::PI)) / (2.0 * std::f64::consts::PI);
                color_for_draw.borrow_mut().set_hsv(
                    map(hue as f32, 0.0, 1.0, 0.0, u16::MAX as f32) as u16,
                    1,
                    1,
                );

                let (h_r, h_g, h_b) = hsv_to_rgb(hue, 1.0, 1.0);

                // Pfad (nur zur Berechnung/Optional)
                cr.move_to(points[0].0, points[0].1);
                cr.line_to(points[1].0, points[1].1);
                cr.line_to(points[2].0, points[2].1);
                cr.close_path();

                // Rastere das Dreieck in eine temporäre ImageSurface mit baryzentrischer Mischung
                let w = width as i32;
                let h = height as i32;
                let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).unwrap();
                let stride = surface.stride() as usize;
                let mut data = surface.data().unwrap(); // <- mutable gemacht

                // Eckfarben: points[0] = Hue, points[1] = Weiß, points[2] = Schwarz
                let c0 = (h_r, h_g, h_b);
                let c1 = (1.0, 1.0, 1.0);
                let c2 = (0.0, 0.0, 0.0);

                let (x0, y0) = points[0];
                let (x1, y1) = points[1];
                let (x2, y2) = points[2];

                // Begrenzungsrechteck
                let min_x = (points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min).floor() as i32).clamp(0, w-1);
                let max_x = (points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max).ceil() as i32).clamp(0, w-1);
                let min_y = (points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min).floor() as i32).clamp(0, h-1);
                let max_y = (points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max).ceil() as i32).clamp(0, h-1);

                let denom = (y1 - y2)*(x0 - x2) + (x2 - x1)*(y0 - y2);

                for py in min_y..=max_y {
                    for px in min_x..=max_x {
                        let fx = px as f64 + 0.5;
                        let fy = py as f64 + 0.5;
                        let w0 = ((y1 - y2)*(fx - x2) + (x2 - x1)*(fy - y2)) / denom;
                        let w1 = ((y2 - y0)*(fx - x2) + (x0 - x2)*(fy - y2)) / denom;
                        let w2 = 1.0 - w0 - w1;
                        if w0 >= -1e-6 && w1 >= -1e-6 && w2 >= -1e-6 {
                            let rr = w0*c0.0 + w1*c1.0 + w2*c2.0;
                            let gg = w0*c0.1 + w1*c1.1 + w2*c2.1;
                            let bb = w0*c0.2 + w1*c1.2 + w2*c2.2;
                            let r8 = (rr * 255.0).clamp(0.0, 255.0) as u32;
                            let g8 = (gg * 255.0).clamp(0.0, 255.0) as u32;
                            let b8 = (bb * 255.0).clamp(0.0, 255.0) as u32;
                            let a8 = 255u32;
                            let pixel = (a8 << 24) | (r8 << 16) | (g8 << 8) | b8;
                            let bytes = pixel.to_ne_bytes();
                            let idx = py as usize * stride + px as usize * 4;
                            data[idx..idx+4].copy_from_slice(&bytes);
                        }
                    }
                }

                drop(data); // <- wichtige Freigabe der mutablen Borrow vor Verwendung von `surface` erneut

                cr.set_source_surface(&surface, 0.0, 0.0).unwrap();
                cr.paint().unwrap();
            });
        }

        // Mausbewegung über EventControllerMotion
        let motion_controller = gtk4::EventControllerMotion::new();
        {
            let mouse_angle = mouse_angle.clone();
            let left_button_pressed = left_button_pressed.clone();
            let drawing_clone = drawing.clone();
            motion_controller.connect_motion(move |_, x, y| {
                if *left_button_pressed.borrow() {
                    let cx = drawing_clone.width() as f64 / 2.0;
                    let cy = drawing_clone.height() as f64 / 2.0;

                    let dx = x - cx;
                    let dy = y - cy;

                    // Nullpunkt oben
                    let angle = dx.atan2(-dy);
                    *mouse_angle.borrow_mut() = angle;
                    drawing_clone.queue_draw();
                }
            });
        }
        drawing.add_controller(motion_controller);

        // Linksklick mit GestureClick
        let click = gtk4::GestureClick::new();
        click.set_button(1); // linker Klick
        {
            let left_button_pressed = left_button_pressed.clone();
            click.connect_pressed(move |_, _, _, _| {
                *left_button_pressed.borrow_mut() = true;
            });
        }
        {
            let left_button_pressed = left_button_pressed.clone();
            click.connect_released(move |_, _, _, _| {
                *left_button_pressed.borrow_mut() = false;
            });
        }
        drawing.add_controller(click);

        drawing.set_visible(true);
        hsv_box.append(&drawing);
        hsv_box.set_visible(true);

        // Stack und Switcher
        stack.add_titled(&rgb_box, Some("rgb"), "RGB");
        stack.add_titled(&cmy_box, Some("cmy"), "CMY");
        stack.add_titled(&hsv_box, Some("hsv"), "HSV");
        stack.set_visible(true);

        let switcher = gtk::StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        root.append(&switcher);
        root.append(&stack);

        Self {root, color,}
    }

    pub fn widget(&self) -> &GtkBox {
        &self.root
    }
}

pub fn map( x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32, ) -> f32 {
    (x - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}

// HSV -> RGB Hilfsfunktion
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
