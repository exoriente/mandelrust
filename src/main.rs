extern crate image as im;
extern crate itertools;
extern crate piston_window;
extern crate rayon;
extern crate vecmath;

use itertools::iproduct;
use piston_window::*;
use rayon::prelude::*;
use vecmath::*;

mod complex;
mod settings;

use complex::Complex;

type Color = [u8; 4];

struct View {
    r: f64,
    i: f64,
    zoom: f64,
    sharpness: u32,
}

impl View {
    fn pixel_to_complex(self: &View, canvas_size: (u32, u32), pixel: (u32, u32)) -> Complex {
        let (width, height) = canvas_size;
        let (x, y) = pixel;
        Complex {
            r: (x as f64 - width as f64 / 2.) / self.zoom + self.r,
            i: (height as f64 / 2. - y as f64) / self.zoom + self.i,
        }
    }

    fn step_left(self: &mut View) {
        self.r -= settings::STEP_SIZE / self.zoom;
    }

    fn step_right(self: &mut View) {
        self.r += settings::STEP_SIZE / self.zoom;
    }

    fn step_up(self: &mut View) {
        self.i += settings::STEP_SIZE / self.zoom;
    }

    fn step_down(self: &mut View) {
        self.i -= settings::STEP_SIZE / self.zoom;
    }

    fn step_zoom_in(self: &mut View) {
        self.zoom *= settings::ZOOM_STEP_SIZE;
    }

    fn step_zoom_out(self: &mut View) {
        self.zoom /= settings::ZOOM_STEP_SIZE;
    }

    fn sharpen(self: &mut View) {
        self.sharpness += 10;
    }

    fn unsharpen(self: &mut View) {
        self.sharpness -= 10;
    }
}

fn circle(c: Complex, iterations: u32) -> i32 {
    let d = (c.r * c.r + c.i * c.i).sqrt();
    if d <= 1. {
        -1
    }
    else {
        (iterations - d.floor() as u32) as i32
    }
}

fn mandelbrot(c: Complex, iterations: u32) -> i32 {
    let mut z = Complex { r: 0., i: 0. };
    for i in 0..iterations {
        z = z * z + c;
        if z.norm() > 2. {
            return i as i32;
        }
    }
    return -1;
}

fn z_to_color(z: i32, steps: u32) -> Color {
    if z == -1 {
        [0, 0, 0, 255]
    } else {
        let red = ((255. / steps as f64) * z as f64) as u8;
        [red, 0, 0, 255]
    }
}

fn draw_fast(view: &View, width: u32, height: u32) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let all_x = 0..width;
    let all_y = 0..height;

    let pixels = iproduct!(all_x, all_y).par_bridge().map(
        |(x, y)| {
            (x, y, 
            z_to_color(
                mandelbrot(
                    view.pixel_to_complex((width, height), (x, y)),
                    view.sharpness,
                ),
                view.sharpness,
            ))
        }
    ).collect::<Vec<_>>();

    let mut canvas = im::ImageBuffer::new(width, height);

    for (x, y, color) in pixels {
        canvas.put_pixel(x, y, im::Rgba(color));
    }

    return canvas
}


fn draw_texture(view: &View, width: u32, height: u32) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let all_x = 0..width;
    let all_y = 0..height;

    let pixels = iproduct!(all_y, all_x).map(
        |(y, x)| {
            z_to_color(
                mandelbrot(
                    view.pixel_to_complex((width, height), (x, y)),
                    view.sharpness,
                ),
                view.sharpness,
            )
        }
    ).flatten().collect();

    im::ImageBuffer::from_raw(width, height, pixels).unwrap()
}


fn main() {
    let opengl = OpenGL::V3_2;
    let (width, height) = (settings::WIDTH, settings::HEIGHT);
    let mut window: PistonWindow = WindowSettings::new(settings::TITLE, (width, height))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .build()
        .unwrap();
    
    let mut view = View {
        r: -0.5,
        i: 0.,
        zoom: 300.,
        sharpness: 30,
    };

    let mut canvas = draw_fast(&view, width, height);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            texture.update(&mut texture_context, &canvas).unwrap();
            window.draw_2d(&e, |c, g, device| {
                // Update texture before rendering.
                texture_context.encoder.flush(device);

                clear([1.0; 4], g);
                image(&texture, c.transform, g);
            });
        }
        if let Some(button) = e.release_args() {
            if button == Button::Keyboard(Key::Left) {
                view.step_left();
                canvas = draw_fast(&view, width, height);
            }
            if button == Button::Keyboard(Key::Right) {
                view.step_right();
                canvas = draw_fast(&view, width, height);
            }
            if button == Button::Keyboard(Key::Up) {
                view.step_up();
                canvas = draw_fast(&view, width, height);
            }
            if button == Button::Keyboard(Key::Down) {
                view.step_down();
                canvas = draw_fast(&view, width, height);
            }
            if button == Button::Keyboard(Key::Z) {
                view.step_zoom_in();
                canvas = draw_fast(&view, width, height);
            }
            if button == Button::Keyboard(Key::A) {
                view.step_zoom_out();
                canvas = draw_fast(&view, width, height);
            }
            if button == Button::Keyboard(Key::X) {
                view.sharpen();
                canvas = draw_fast(&view, width, height);
            }
            if button == Button::Keyboard(Key::S) {
                view.unsharpen();
                canvas = draw_fast(&view, width, height);
            }
        };
    }
}
