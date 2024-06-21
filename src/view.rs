use super::complex::Complex;
use super::settings;

pub struct View {
    pub r: f64,
    pub i: f64,
    pub zoom: f64,
    pub sharpness: u32,
}

impl View {
    pub fn pixel_to_complex(self: &View, canvas_size: (u32, u32), pixel: (u32, u32)) -> Complex {
        let (width, height) = canvas_size;
        let (x, y) = pixel;
        Complex {
            r: (x as f64 - width as f64 / 2.) / self.zoom + self.r,
            i: (height as f64 / 2. - y as f64) / self.zoom + self.i,
        }
    }

    pub fn step_left(self: &View) -> View {
        View {
            r: self.r - settings::STEP_SIZE / self.zoom,
            ..*self
        }
    }

    pub fn step_right(self: &View) -> View {
        View {
            r: self.r + settings::STEP_SIZE / self.zoom,
            ..*self
        }
    }

    pub fn step_up(self: &View) -> View {
        View {
            i: self.i + settings::STEP_SIZE / self.zoom,
            ..*self
        }
    }

    pub fn step_down(self: &View) -> View {
        View {
            i: self.i - settings::STEP_SIZE / self.zoom,
            ..*self
        }
    }

    pub fn step_zoom_in(self: &View) -> View {
        View {
            zoom: self.zoom * settings::ZOOM_STEP_SIZE,
            ..*self
        }
    }

    pub fn step_zoom_out(self: &View) -> View {
        View {
            zoom: self.zoom / settings::ZOOM_STEP_SIZE,
            ..*self
        }
    }

    pub fn zoom_by(self: &View, zoom_factor: f64) -> View {
        View {
            zoom: self.zoom * zoom_factor,
            ..*self
        }
    }

    pub fn sharpen(self: &View) -> View {
        View {
            sharpness: self.sharpness + 10,
            ..*self
        }
    }

    pub fn unsharpen(self: &View) -> View {
        View {
            sharpness: self.sharpness - 10,
            ..*self
        }
    }

    pub fn center_on(self: &View, c: Complex) -> View {
        View {
            r: c.r,
            i: c.i,
            ..*self
        }
    }
}
