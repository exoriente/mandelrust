extern crate image as im;
extern crate itertools;
extern crate nonempty;
extern crate piston_window;
extern crate rayon;

use itertools::{iproduct, Itertools};
use nonempty::nonempty;
use piston_window::*;
use rayon::prelude::*;

mod all_between;
mod complex;
mod settings;
mod view;

use all_between::AllBetween;
use complex::Complex;
use view::View;

type Color = [u8; 4];

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
        [red, 0, 0, 255]
    }
}

fn draw_image(
    view: &View,
    width: u32,
    height: u32,
    function: fn(Complex, u32) -> i32,
) -> im::RgbaImage {
    let all_x = 0..width;
    let all_y = 0..height;

    let pixels = iproduct!(all_y, all_x)
        .collect_vec()
        .par_iter()
        .map(|(y, x)| {
            z_to_color(
                function(
                    view.pixel_to_complex((width, height), (*x, *y)),
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

    let mut views = nonempty![View {
        r: -0.75,
        i: 0.,
        zoom: 300.,
        sharpness: 30,
    }];

    let function: fn(Complex, u32) -> i32 = if settings::FUNCTION == "CIRCLE" {
        circle
    } else {
        mandelbrot
    };

    let draw = |view: &View| draw_image(view, width, height, function);

    let mut base_canvas = draw(views.last());
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
                    views.push(views.last().step_left());
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Right) {
                    views.push(views.last().step_right());
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Up) {
                    views.push(views.last().step_up());
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Down) {
                    views.push(views.last().step_down());
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Z) {
                    views.push(views.last().step_zoom_in());
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::A) {
                    views.push(views.last().step_zoom_out());
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::X) {                    
                    let new_view = views.last().sharpen();
                    if views.len() > 1 {
                        views.pop();
                    }
                    views.push(new_view);
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::S) {
                    let new_view = views.last().unsharpen();
                    if views.len() > 1 {
                        views.pop();
                    }
                    views.push(new_view);
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Backspace) {
                    views.pop();
                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                }
                if button == Button::Keyboard(Key::Q) {
                    break;
                }
                if button == Button::Keyboard(Key::F1) {
                    println!("Zoom factor: {}", views.last().zoom);
                    println!("Iterations: {}", views.last().sharpness);
                }
            }
            if button == Button::Mouse(MouseButton::Left) {
                if press_position == Some(mouse_position) {
                    let c = views.last().pixel_to_complex(
                        (width, height),
                        (mouse_position.0 as u32, mouse_position.1 as u32),
                    );
                    views.push(views.last().center_on(c));
                    base_canvas = draw(views.last());

                    canvas = &base_canvas;
                    press_position = None;
                } else if let Some(base_pos) = press_position {
                    let (x1, y1) = (base_pos.0 as u32, base_pos.1 as u32);
                    let (x2, y2) = (mouse_position.0 as u32, mouse_position.1 as u32);

                    let selected_width = x1.abs_diff(x2);
                    let selected_height = y1.abs_diff(y2);

                    let new_center = views
                        .last()
                        .pixel_to_complex((width, height), ((x1 + x2) / 2, (y1 + y2) / 2));

                    let zoom_factor_x = width as f64 / selected_width as f64;
                    let zoom_factor_y = height as f64 / selected_height as f64;
                    let zoom_factor = if zoom_factor_x <= zoom_factor_y {
                        zoom_factor_x
                    } else {
                        zoom_factor_y
                    };
                    views.push(views.last().center_on(new_center).zoom_by(zoom_factor));

                    base_canvas = draw(views.last());
                    canvas = &base_canvas;
                    press_position = None;
                }
            }
            if button == Button::Mouse(MouseButton::Right) {
                if press_position.is_none() {
                    let c = views.last().pixel_to_complex(
                        (width, height),
                        (mouse_position.0 as u32, mouse_position.1 as u32),
                    );
                    views.push(views.last().center_on(c).step_zoom_out());
                    base_canvas = draw(views.last());
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
                    for x in x1.all_between(x2) {
                        overlay.put_pixel(x, y1, im::Rgba([192, 192, 192, 255]));
                        overlay.put_pixel(x, y2, im::Rgba([192, 192, 192, 255]));
                    }
                    for y in y1.all_between(y2) {
                        overlay.put_pixel(x1, y, im::Rgba([192, 192, 192, 255]));
                        overlay.put_pixel(x2, y, im::Rgba([192, 192, 192, 255]));
                    }
                    canvas = &overlay;
                }
            }
        }
    }
}
