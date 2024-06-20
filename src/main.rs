extern crate image as im;
extern crate itertools;
extern crate piston_window;
extern crate rayon;
extern crate vecmath;

use itertools::iproduct;
use piston_window::*;
use rayon::prelude::*;
use std::{cmp::{max, min}, str::FromStr};

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

    fn zoom_by(self: &mut View, zoom_factor: f64) {
        self.zoom *= zoom_factor;
    }

    fn sharpen(self: &mut View) {
        self.sharpness += 10;
    }

    fn unsharpen(self: &mut View) {
        self.sharpness -= 10;
    }

    fn center_on(self: &mut View, c: Complex) {
        self.r = c.r;
        self.i = c.i;
    }
}

fn circle(c: Complex, iterations: u32) -> i32 {
    let d = (c.r * c.r + c.i * c.i).sqrt();
    if d <= 1. {
        -1
    } else {
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
        [red, red, red, 255]
    }
}

fn draw_fast(view: &View, width: u32, height: u32) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let all_x = 0..width;
    let all_y = 0..height;

    let pixels = iproduct!(all_x, all_y)
        .par_bridge()
        .map(|(x, y)| {
            (
                x,
                y,
                z_to_color(
                    mandelbrot(
                        view.pixel_to_complex((width, height), (x, y)),
                        view.sharpness,
                    ),
                    view.sharpness,
                ),
            )
        })
        .collect::<Vec<_>>();

    let mut canvas = im::ImageBuffer::new(width, height);

    for (x, y, color) in pixels {
        canvas.put_pixel(x, y, im::Rgba(color));
    }

    return canvas;
}

fn draw_texture(view: &View, width: u32, height: u32) -> im::ImageBuffer<im::Rgba<u8>, Vec<u8>> {
    let all_x = 0..width;
    let all_y = 0..height;

    let pixels = iproduct!(all_y, all_x)
        .map(|(y, x)| {
            z_to_color(
                mandelbrot(
                    view.pixel_to_complex((width, height), (x, y)),
                    view.sharpness,
                ),
                view.sharpness,
            )
        })
        .flatten()
        .collect();

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

    let mut base_canvas = draw_fast(&view, width, height);
    let mut overlay = base_canvas.clone();
    let mut canvas = &base_canvas;

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };
    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &overlay, &TextureSettings::new()).unwrap();

    let mut mouse_position = (0f32, 0f32);
    let mut press_position: Option<(f32, f32)> = None;

    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            texture.update(&mut texture_context, canvas).unwrap();
            window.draw_2d(&e, |c, g, device| {
                // Update texture before rendering.
                texture_context.encoder.flush(device);

                clear([1.0; 4], g);
                image(&texture, c.transform, g);
            });
        }
        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            press_position = Some(mouse_position);
        }
        if let Some(button) = e.release_args() {
            if press_position == None {
                if button == Button::Keyboard(Key::Left) {
                    view.step_left();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Right) {
                    view.step_right();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Up) {
                    view.step_up();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Down) {
                    view.step_down();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Z) {
                    view.step_zoom_in();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::A) {
                    view.step_zoom_out();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::X) {
                    view.sharpen();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::S) {
                    view.unsharpen();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Q) {
                    break;
                }
                if button == Button::Keyboard(Key::F1) {
                    println!("Zoom factor: {}", view.zoom);
                    println!("Iterations: {}", view.sharpness);
                }
            }
            if button == Button::Mouse(MouseButton::Left) {
                if press_position == Some(mouse_position) {
                    let c = view.pixel_to_complex(
                        (width, height),
                        (mouse_position.0 as u32, mouse_position.1 as u32),
                    );
                    view.center_on(c);
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                    press_position = None;
                } else if let Some(base_pos) = press_position {
                    let (x1, y1) = (base_pos.0 as u32, base_pos.1 as u32);
                    let (x2, y2) = (mouse_position.0 as u32, mouse_position.1 as u32);

                    let selected_width = max(x1, x2) - min(x1, x2);
                    let selected_height = max(y1, y2) - min(y1, y2);

                    let new_center =
                        view.pixel_to_complex((width, height), ((x1 + x2) / 2, (y1 + y2) / 2));
                    view.center_on(new_center);

                    let zoom_factor_x = width as f64 / selected_width as f64;
                    let zoom_factor_y = height as f64 / selected_height as f64;
                    let zoom_factor = if zoom_factor_x <= zoom_factor_y {
                        zoom_factor_x
                    } else {
                        zoom_factor_y
                    };
                    view.zoom_by(zoom_factor);

                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                    press_position = None;
                }
            }
            if button == Button::Mouse(MouseButton::Right) {
                if press_position.is_none() {
                    let c = view.pixel_to_complex(
                        (width, height),
                        (mouse_position.0 as u32, mouse_position.1 as u32),
                    );
                    view.center_on(c);
                    view.step_zoom_out();
                    base_canvas = draw_fast(&view, width, height);
                    canvas = &base_canvas;
                    press_position = None;
                }
            }
        };
        if e.mouse_cursor_args().is_some() {
            if let Some(pos) = e.mouse_cursor_args() {
                mouse_position = (pos[0] as f32, pos[1] as f32);
            }
            if let Some(base_pos) = press_position {
                if base_pos != mouse_position {
                    let (x1, y1) = (base_pos.0 as u32, base_pos.1 as u32);
                    let (x2, y2) = (mouse_position.0 as u32, mouse_position.1 as u32);

                    overlay = base_canvas.clone();
                    for x in min(x1, x2)..max(x1, x2) {
                        overlay.put_pixel(x, y1, im::Rgba([192, 192, 192, 255]));
                        overlay.put_pixel(x, y2, im::Rgba([192, 192, 192, 255]));
                    }
                    for y in min(y1, y2)..max(y1, y2) {
                        overlay.put_pixel(x1, y, im::Rgba([192, 192, 192, 255]));
                        overlay.put_pixel(x2, y, im::Rgba([192, 192, 192, 255]));
                    }
                    canvas = &overlay;
                }
            }
        }
    }
}