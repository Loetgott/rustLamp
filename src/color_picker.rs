// language: rust
use gtk4::prelude::*;
use gtk4::{self as gtk, cairo, Box as GtkBox, DrawingArea, Stack};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use crate::color::Color;

pub struct ColorPicker {
    root: GtkBox,
    color: Rc<RefCell<Color>>,
}

impl ColorPicker {
    pub fn new(color: Rc<RefCell<Color>>) -> Self {
        let color_rc = Rc::clone(&color);

        let circle_radius = Rc::new(Cell::new(0.0_f64));
        let mouse_angle = Rc::new(RefCell::new(0.0_f64));
        let left_button_pressed = Rc::new(RefCell::new(false));

        let root = GtkBox::new(gtk::Orientation::Vertical, 8);
        let stack = Stack::new();

        let rgb_box = GtkBox::new(gtk::Orientation::Horizontal, 4);
        rgb_box.set_visible(true);
        let cmy_box = GtkBox::new(gtk::Orientation::Horizontal, 4);
        cmy_box.set_visible(true);
        let hsv_box = GtkBox::new(gtk::Orientation::Horizontal, 4);

        let drawing = DrawingArea::new();
        drawing.set_content_width(500);
        drawing.set_content_height(500);

        let point_a = Rc::new(RefCell::new((0.0_f64, 0.0_f64)));
        let point_b = Rc::new(RefCell::new((0.0_f64, 0.0_f64)));
        let point_c = Rc::new(RefCell::new((0.0_f64, 0.0_f64)));

        // ⚡ Punkt-Klone für Draw- und Click-Handler getrennt
        let point_a_for_draw = point_a.clone();
        let point_b_for_draw = point_b.clone();
        let point_c_for_draw = point_c.clone();

        let point_a_for_click = point_a.clone();
        let point_b_for_click = point_b.clone();
        let point_c_for_click = point_c.clone();

        // Draw-Funktion
        {
            let mouse_angle = mouse_angle.clone();
            let color_for_draw = color_rc.clone();
            let circle_radius = circle_radius.clone();
            drawing.set_draw_func(move |_, cr, width, height| {
                let cx = width as f64 / 2.0;
                let cy = height as f64 / 2.0;
                let circle_width = 10.0;
                let radius = cx.min(cy) - circle_width;
                circle_radius.set(radius - circle_width / 1.50);

                let offset = -std::f64::consts::PI / 2.0;
                for i in 0..360 {
                    let angle1 = (i as f64).to_radians() + offset - 0.005;
                    let angle2 = ((i + 1) as f64).to_radians() + offset;
                    let (r, g, b) = hsv_to_rgb(i as f64 / 360.0, 1.0, 1.0);
                    cr.set_source_rgb(r, g, b);
                    cr.set_line_width(circle_width * 2.0);
                    cr.arc(cx, cy, radius, angle1, angle2);
                    cr.stroke().unwrap();
                }

                let triangle_radius = radius - circle_width;
                let triangle_angle = 2.0 * std::f64::consts::PI / 3.0;
                let angle_offset = *mouse_angle.borrow();

                let mut points = Vec::with_capacity(3);
                for j in 0..3 {
                    let angle = -std::f64::consts::PI / 2.0 + j as f64 * triangle_angle + angle_offset;
                    let x = cx + triangle_radius * angle.cos();
                    let y = cy + triangle_radius * angle.sin();
                    points.push((x, y));
                }

                // Punkte in Draw-RefCells schreiben
                *point_a_for_draw.borrow_mut() = points[0];
                *point_b_for_draw.borrow_mut() = points[1];
                *point_c_for_draw.borrow_mut() = points[2];

                let angle = -*mouse_angle.borrow();
                let hue = ((-angle).rem_euclid(2.0 * std::f64::consts::PI)) / (2.0 * std::f64::consts::PI);

                color_rc.borrow_mut().set_hue(
                    map(hue as f32, 0.0, 1.0, 0.0, u16::MAX as f32) as u16,
                );

                let (h_r, h_g, h_b) = hsv_to_rgb(hue, 1.0, 1.0);

                cr.move_to(points[0].0, points[0].1);
                cr.line_to(points[1].0, points[1].1);
                cr.line_to(points[2].0, points[2].1);
                cr.close_path();

                let w = width as i32;
                let h = height as i32;
                let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).unwrap();
                let stride = surface.stride() as usize;
                let mut data = surface.data().unwrap();

                let c0 = (h_r, h_g, h_b);
                let c1 = (1.0, 1.0, 1.0);
                let c2 = (0.0, 0.0, 0.0);

                let (x0, y0) = points[0];
                let (x1, y1) = points[1];
                let (x2, y2) = points[2];

                let min_x = (points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min).floor() as i32).clamp(0, w - 1);
                let max_x = (points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max).ceil() as i32).clamp(0, w - 1);
                let min_y = (points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min).floor() as i32).clamp(0, h - 1);
                let max_y = (points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max).ceil() as i32).clamp(0, h - 1);

                let denom = (y1 - y2) * (x0 - x2) + (x2 - x1) * (y0 - y2);

                for py in min_y..=max_y {
                    for px in min_x..=max_x {
                        let fx = px as f64 + 0.5;
                        let fy = py as f64 + 0.5;
                        let w0 = ((y1 - y2) * (fx - x2) + (x2 - x1) * (fy - y2)) / denom;
                        let w1 = ((y2 - y0) * (fx - x2) + (x0 - x2) * (fy - y2)) / denom;
                        let w2 = 1.0 - w0 - w1;
                        if w0 >= -1e-6 && w1 >= -1e-6 && w2 >= -1e-6 {
                            let rr = w0 * c0.0 + w1 * c1.0 + w2 * c2.0;
                            let gg = w0 * c0.1 + w1 * c1.1 + w2 * c2.1;
                            let bb = w0 * c0.2 + w1 * c1.2 + w2 * c2.2;
                            let r8 = (rr * 255.0).clamp(0.0, 255.0) as u32;
                            let g8 = (gg * 255.0).clamp(0.0, 255.0) as u32;
                            let b8 = (bb * 255.0).clamp(0.0, 255.0) as u32;
                            let a8 = 255u32;
                            let pixel = (a8 << 24) | (r8 << 16) | (g8 << 8) | b8;
                            let bytes = pixel.to_ne_bytes();
                            let idx = py as usize * stride + px as usize * 4;
                            data[idx..idx + 4].copy_from_slice(&bytes);
                        }
                    }
                }

                drop(data);
                cr.set_source_surface(&surface, 0.0, 0.0).unwrap();
                cr.paint().unwrap();
            });
        }
        let pa = point_a_for_click.clone();
        let pb = point_b_for_click.clone();
        let pc = point_c_for_click.clone();

        // Mausbewegung etc. bleibt unverändert...
        // Klick-Handler nutzt nun die _for_click Klone
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
                    let angle = dx.atan2(-dy);
                    *mouse_angle.borrow_mut() = angle;
                    drawing_clone.queue_draw();

                    let pa = pa.borrow();
                    let pb = pb.borrow();
                    let pc = pc.borrow();

                    let distance_a = calculate_distance((x,y),(pa.0,pa.1));//((x - pa.0).powi(2) + (y - pa.1).powi(2)).sqrt();
                    let distance_b = calculate_distance((x,y),(pb.0,pb.1));//((x - pb.0).powi(2) + (y - pb.1).powi(2)).sqrt();
                    let distance_c = calculate_distance((x,y),(pc.0,pc.1));//((x - pc.0).powi(2) + (y - pc.1).powi(2)).sqrt();

                    //println!("{}", distance_a);
                    //println!("{}", distance_b);
                    //println!("{}", distance_c);
                }
            });
        }
        drawing.add_controller(motion_controller);

        let click = gtk4::GestureClick::new();
        click.set_button(1);

        let drawing_for_pressed = drawing.clone();
        let drawing_for_released = drawing.clone();

        let pa = point_a_for_click.clone();
        let pb = point_b_for_click.clone();
        let pc = point_c_for_click.clone();

        let left_button_pressed_pressed = left_button_pressed.clone();
        let left_button_pressed_released = left_button_pressed.clone();

        let mouse_angle_pressed = mouse_angle.clone();
        let mouse_angle_released = mouse_angle.clone();

        let circle_radius_pressed = circle_radius.clone();
        let circle_radius_released = circle_radius.clone();

        click.connect_pressed(move |_, _, x, y| {
            let cx = drawing_for_pressed.width() as f64 / 2.0;
            let cy = drawing_for_pressed.height() as f64 / 2.0;
            let dx = x - cx;
            let dy = y - cy;
            let distance = (dx*dx + dy*dy).sqrt();
            let angle = dx.atan2(-dy);

            if distance >= circle_radius_pressed.get() {
                *mouse_angle_pressed.borrow_mut() = angle;
                *left_button_pressed_pressed.borrow_mut() = true;
            }

            let pa = pa.borrow();
            let pb = pb.borrow();
            let pc = pc.borrow();

            let distance_a = calculate_distance((x,y),(pa.0,pa.1));//((x - pa.0).powi(2) + (y - pa.1).powi(2)).sqrt();
            let distance_b = calculate_distance((x,y),(pb.0,pb.1));//((x - pb.0).powi(2) + (y - pb.1).powi(2)).sqrt();
            let distance_c = calculate_distance((x,y),(pc.0,pc.1));//((x - pc.0).powi(2) + (y - pc.1).powi(2)).sqrt();

            //println!("{}", distance_a);
            //println!("{}", distance_b);
            //println!("{}", distance_c);
            //println!("---");

            drawing_for_pressed.queue_draw();
            //println!("Abstand zur Mitte: {:.2}", distance);
        });

        let pa = point_a_for_click.clone();
        let pb = point_b_for_click.clone();
        let pc = point_c_for_click.clone();

        click.connect_released(move |_, _, x, y| {
            let cx = drawing_for_released.width() as f64 / 2.0;
            let cy = drawing_for_released.height() as f64 / 2.0;
            let dx = x - cx;
            let dy = y - cy;
            let distance = (dx*dx + dy*dy).sqrt();
            let angle = dx.atan2(-dy);

            if distance >= circle_radius_released.get() || *left_button_pressed_released.borrow() {
                *mouse_angle_released.borrow_mut() = angle;
                *left_button_pressed_released.borrow_mut() = false;
            }

            let pa = pa.borrow();
            let pb = pb.borrow();
            let pc = pc.borrow();

            let distance_a = calculate_distance((x,y),(pa.0,pa.1));//((x - pa.0).powi(2) + (y - pa.1).powi(2)).sqrt();
            let distance_b = calculate_distance((x,y),(pb.0,pb.1));//((x - pb.0).powi(2) + (y - pb.1).powi(2)).sqrt();
            let distance_c = calculate_distance((x,y),(pc.0,pc.1));//((x - pc.0).powi(2) + (y - pc.1).powi(2)).sqrt();

            //println!("A: {}", distance_a);
            //println!("B: {}", distance_b);
            //println!("C: {}", distance_c);
            //println!("---");

            drawing_for_released.queue_draw();
            //println!("Losgelassen, Abstand: {:.2}", distance);
        });
        drawing.add_controller(click);

        let drag = gtk4::GestureDrag::new();

        let drag_start = Rc::new(RefCell::new((0.0_f64, 0.0_f64)));

        drag.connect_drag_begin({
            let drag_start = drag_start.clone();
            move |_, x, y| {
                // x,y sind HIER absolut
                *drag_start.borrow_mut() = (x, y);
            }
        });

        drag.connect_drag_update({
            let drag = gtk4::GestureDrag::new();

            let drag_start = drag_start.clone();
            let pa = point_a_for_click.clone();
            let pb = point_b_for_click.clone();
            let pc = point_c_for_click.clone();
            let color_rc_clone = color_rc.clone(); // 🔹 Clone vor der Closure

            drag.connect_drag_update(move |_, dx, dy| {
                // ======================
                // Drag-Position
                // ======================
                let (sx, sy) = *drag_start.borrow();
                let x = sx + dx;
                let y = sy + dy;

                // ======================
                // Punkte EINMAL borrowen
                // ======================
                let (pax, pay) = *pa.borrow();
                let (pbx, pby) = *pb.borrow();
                let (pcx, pcy) = *pc.borrow();

                const MAX_ANGLE: f64 = std::f64::consts::PI / 3.0;

                // VALUE
                let vec_b_a = (x - pbx, y - pby);
                let vec_b_c = (pcx - pbx, pcy - pby);
                let dot_val = vec_b_a.0 * vec_b_c.0 + vec_b_a.1 * vec_b_c.1;
                let len_ba = calculate_distance((x, y), (pbx, pby));
                let len_bc = calculate_distance((pbx, pby), (pcx, pcy));
                if len_ba < 1e-6 || len_bc < 1e-6 { return; }
                let mut cos_val = dot_val / (len_ba * len_bc);
                cos_val = cos_val.clamp(-1.0, 1.0);
                let value_angle = cos_val.acos();
                let value = (value_angle / MAX_ANGLE).clamp(0.0, 1.0);

                // SATURATION
                let vec_c_a = (x - pcx, y - pcy);
                let vec_c_b = (pbx - pcx, pby - pcy);
                let dot_sat = vec_c_a.0 * vec_c_b.0 + vec_c_a.1 * vec_c_b.1;
                let len_ca = calculate_distance((x, y), (pcx, pcy));
                let len_cb = calculate_distance((pbx, pby), (pcx, pcy));
                if len_ca < 1e-6 || len_cb < 1e-6 { return; }
                let mut cos_sat = dot_sat / (len_ca * len_cb);
                cos_sat = cos_sat.clamp(-1.0, 1.0);
                let saturation_angle = cos_sat.acos();
                let saturation = (saturation_angle / MAX_ANGLE).clamp(0.0, 1.0);

                // COLOR
                {
                    let mut c = color_rc_clone.borrow_mut(); // 🔹 direkt clone benutzen
                    c.set_saturation(map(saturation as f32, 0.0, 1.0, 0.0, u16::MAX as f32) as u16);
                    c.set_value(map(value as f32, 0.0, 1.0, 0.0, u16::MAX as f32) as u16);
                }

                println!("value      : {:.4}", value);
                println!("saturation : {:.4}", saturation);
                println!("---");
            });

            drawing.add_controller(drag);

        });


        drawing.add_controller(drag);

        drawing.set_visible(true);
        hsv_box.append(&drawing);
        hsv_box.set_visible(true);

        stack.add_titled(&rgb_box, Some("rgb"), "RGB");
        stack.add_titled(&cmy_box, Some("cmy"), "CMY");
        stack.add_titled(&hsv_box, Some("hsv"), "HSV");
        stack.set_visible(true);

        let switcher = gtk::StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        root.append(&switcher);
        root.append(&stack);

        Self {root, color}
    }

    pub fn widget(&self) -> &GtkBox {
        &self.root
    }
}

pub fn map(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (x - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}

fn calculate_distance(p1: (f64, f64), p2: (f64, f64)) -> f64 {
    let dx = p2.0 - p1.0;
    let dy = p2.1 - p1.1;
    (dx * dx + dy * dy).sqrt()
}


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