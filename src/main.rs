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

fn draw(view: &View, canvas: &mut im::ImageBuffer<im::Rgba<u8>, Vec<u8>>) {
    let width = canvas.width();
    let height = canvas.height();

    let all_x = 0..width;
    let all_y = 0..height;

    let pixels = iproduct!(all_x, all_y).par_bridge().map(
        |(x, y)| {
            (x, y, 
            z_to_color(
                circle(
                    view.pixel_to_complex((width, height), (x, y)),
                    view.sharpness,
                ),
                view.sharpness,
            ))
        }
    ).collect::<Vec<_>>();

    for (x, y, color) in pixels {
        canvas.put_pixel(x, y, im::Rgba(color));
    }
}


fn draw_fast(view: &View, width: u32, height: u32) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let all_x = 0..width;
    let all_y = 0..height;

    let pixels = iproduct!(all_x, all_y).map(
        |(x, y)| {
            z_to_color(
                circle(
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

    let mut canvas = im::ImageBuffer::new(width, height);
    let mut redraw = false;
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    let mut last_pos: Option<[f64; 2]> = None;

    let mut view = View {
        r: -0.5,
        i: 0.,
        zoom: 300.,
        sharpness: 30,
    };

    draw(&view, &mut canvas);

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
        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                redraw = true;
            }
        };
        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                redraw = false;
                last_pos = None
            }
            if button == Button::Keyboard(Key::Left) {
                view.step_left();
                draw(&view, &mut canvas);
            }
            if button == Button::Keyboard(Key::Right) {
                view.step_right();
                draw(&view, &mut canvas);
            }
            if button == Button::Keyboard(Key::Up) {
                view.step_up();
                draw(&view, &mut canvas);
            }
            if button == Button::Keyboard(Key::Down) {
                view.step_down();
                draw(&view, &mut canvas);
            }
            if button == Button::Keyboard(Key::Z) {
                view.step_zoom_in();
                draw(&view, &mut canvas);
            }
            if button == Button::Keyboard(Key::A) {
                view.step_zoom_out();
                draw(&view, &mut canvas);
            }
            if button == Button::Keyboard(Key::X) {
                view.sharpen();
                draw(&view, &mut canvas);
            }
            if button == Button::Keyboard(Key::S) {
                view.unsharpen();
                draw(&view, &mut canvas);
            }
        };
        if redraw {
            if let Some(pos) = e.mouse_cursor_args() {
                let (x, y) = (pos[0] as f32, pos[1] as f32);

                if let Some(p) = last_pos {
                    let (last_x, last_y) = (p[0] as f32, p[1] as f32);
                    let distance = vec2_len(vec2_sub(p, pos)) as u32;

                    for i in 0..distance {
                        let diff_x = x - last_x;
                        let diff_y = y - last_y;
                        let delta = i as f32 / distance as f32;
                        let new_x = (last_x + (diff_x * delta)) as u32;
                        let new_y = (last_y + (diff_y * delta)) as u32;
                        if new_x < width && new_y < height {
                            canvas.put_pixel(new_x, new_y, im::Rgba([255, 0, 0, 255]));
                        };
                    }
                };

                last_pos = Some(pos)
            };
        }
    }
}
