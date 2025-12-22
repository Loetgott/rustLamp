use std::cmp::{max, min};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: u16,
    pub green: u16,
    pub blue: u16,

    pub cyan: u16,
    pub magenta: u16,
    pub yellow: u16,

    pub hue: u16,
    pub saturation: u16,
    pub value: u16,
}

impl Color {
    pub fn new() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            cyan: u16::MAX,
            magenta: u16::MAX,
            yellow: u16::MAX,
            hue: 0,
            saturation: 0,
            value: 0,
        }
    }

    pub fn set_rgb(&mut self, red: u16, green: u16, blue: u16) {
        self.red = red;
        self.green = green;
        self.blue = blue;

        /* --- CMY --- */
        self.cyan = u16::MAX - red;
        self.magenta = u16::MAX - green;
        self.yellow = u16::MAX - blue;

        /* --- HSV --- */
        let max_v = max(red, max(green, blue));
        let min_v = min(red, min(green, blue));
        let delta = max_v - min_v;

        self.value = max_v;

        self.saturation = if max_v == 0 {
            0
        } else {
            ((delta as u32 * u16::MAX as u32) / max_v as u32) as u16
        };

        let mut h: f32 = if delta == 0 {
            0.0
        } else if max_v == red {
            (green as f32 - blue as f32) / delta as f32
        } else if max_v == green {
            (blue as f32 - red as f32) / delta as f32 + 2.0
        } else {
            (red as f32 - green as f32) / delta as f32 + 4.0
        };

        if h < 0.0 {
            h += 6.0;
        }

        self.hue = (h * (u16::MAX as f32 / 6.0)) as u16;
    }

    pub fn set_hsv(&mut self, hue: u16, saturation: u16, value: u16) {
        self.hue = hue;
        self.saturation = saturation;
        self.value = value;
        //println!("HSV: {}, {}, {}", hue, saturation, value);

        let h = hue as f32 * 6.0 / u16::MAX as f32;
        let s = saturation as f32 / u16::MAX as f32;
        let v = value as f32 / u16::MAX as f32;

        let c = v * s;
        let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
        let m = v - c;

        let (r1, g1, b1) = match h {
            h if h < 1.0 => (c, x, 0.0),
            h if h < 2.0 => (x, c, 0.0),
            h if h < 3.0 => (0.0, c, x),
            h if h < 4.0 => (0.0, x, c),
            h if h < 5.0 => (x, 0.0, c),
            _                 => (c, 0.0, x),
        };

        self.red   = ((r1 + m) * u16::MAX as f32) as u16;
        self.green = ((g1 + m) * u16::MAX as f32) as u16;
        self.blue  = ((b1 + m) * u16::MAX as f32) as u16;

        self.cyan = u16::MAX - self.red;
        self.magenta = u16::MAX - self.green;
        self.yellow = u16::MAX - self.blue;
        //println!("RGB: {}, {}, {}", self.red, self.green, self.blue);
    }

    pub fn map_rgb_to_unit(&self) -> (f32, f32, f32) {
        (
            u16_to_unit(self.red),
            u16_to_unit(self.green),
            u16_to_unit(self.blue),
        )
    }

    pub fn map_hsv_to_unit(&self) -> (f32, f32, f32) {
        (
            u16_to_unit(self.hue),
            u16_to_unit(self.saturation),
            u16_to_unit(self.value),
        )
    }
}

#[inline]
fn u16_to_unit(x: u16) -> f32 {
    x as f32 / u16::MAX as f32
}


