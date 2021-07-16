extern crate sdl2;
extern crate openweathermap;

// SDL libs
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::render::TextureQuery;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use super::StateMutex;
use std::sync::{Mutex, Arc, Condvar};

use openweathermap::blocking::weather;

const INTENT_DONE : &str = "done";

/// Hijacks an SDL context and displays the weather
pub fn show_weather<T: crate::main_loop::sdl2::render::RenderTarget, U>(res_width : u32, res_height : u32,
                                                                    key : &str, loc : &str, country : &str, canvas : &mut Canvas<T>, 
                                                                    ttf_context : &sdl2::ttf::Sdl2TtfContext, 
                                                                    texture_creator : &TextureCreator<U>, 
                                                                    state: StateMutex, 
                                                                    new_intent_available : Arc<(Mutex<bool>,Condvar)>) 
                                                                    -> Result<(), String> {
    let mut parsed_intent = "".to_owned();

    let (lock, cvar) = &*new_intent_available;

    // Load the font
    let mut font = ttf_context.load_font("assets/font.ttf", 30)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let weather = match weather(format!("{},{}", loc, country).as_str(), "metric", "en", key) {
        Ok(current) => current,
        Err(e) => return Err(e),
    };

    while parsed_intent != INTENT_DONE {
        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(format!("weather in {}: {}", weather.name, weather.weather[0].description).as_str())
            .blended(Color::RGBA(0, 0, 0, 255))
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGBA(128, 230, 209, 1));
        canvas.clear();

        let TextureQuery { width, height, .. } = texture.query();

        // From https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/ttf-demo.rs
        // If the example text is too big for the screen, downscale it (and center it regardless)
        let padding = 5;
        let target = get_centered_rect(
            res_width,
            res_height,
            width,
            height,
            res_width - padding,
            res_height - padding,
        );

        canvas.copy(&texture, None, Some(target))?;
        canvas.present();

        // Wait while there isn't a new intent for auxiliary functions
        let mut new = cvar.wait_while(lock.lock().unwrap(), |new| !*new).unwrap();
        *new = false;

        parsed_intent = state.lock().unwrap().current_intent.clone();
    }   

    Ok(())
}


// https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/ttf-demo.rs
// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

// https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/ttf-demo.rs
// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(res_width : u32, res_height : u32, rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (res_width as i32 - w) / 2;
    let cy = (res_height as i32 - h) / 2;
    rect!(cx, cy, w, h)
}
