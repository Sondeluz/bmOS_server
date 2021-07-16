extern crate sdl2;
extern crate random_number; // For loading random assets for a given intent

// SDL libs
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::render::TextureQuery;
use sdl2::rect::Rect;
use sdl2::render::TextureCreator;
use std::time::Duration; // Sleeping and timers
use std::path::{Path}; // for providing paths to SDL
use super::StateMutex;
use std::sync::{Mutex, Arc, Condvar};

use crate::main_loop::sdl2::image::LoadTexture; // use the implemented trait here
use super::play_sound_blocking;

const INTENT_5_MORE : &str = "5more";
const INTENT_10_MORE : &str = "10more";
const INTENT_20_MORE : &str = "20more";
const INTENT_5_LESS : &str = "5less";
const INTENT_10_LESS : &str = "10less";
const INTENT_20_LESS : &str = "20less";
const INTENT_DONE : &str = "done";

const ALARM_FACE : &str = "assets/faces/alarm.jpg";
const ALARM_SOUND : &str = "assets/audio/alarm.wav";


/// Hijacks an SDL context and displays a Duration while the received intent is not INTENT_DONE,
/// which is modified by change_duration and eventually returned
pub fn get_time<T: crate::main_loop::sdl2::render::RenderTarget, U>(res_width : u32, res_height : u32,
                                                                    canvas : &mut Canvas<T>,
                                                                    ttf_context : &sdl2::ttf::Sdl2TtfContext, 
                                                                    texture_creator : &TextureCreator<U>, 
                                                                    state: StateMutex, 
                                                                    new_intent_available : Arc<(Mutex<bool>,Condvar)>) 
                                                                    -> Result<Duration, String> {
    let mut parsed_duration = Duration::new(0, 0);
    let mut parsed_intent = "".to_owned();

    let (lock, cvar) = &*new_intent_available;

    // Load the font
    let mut font = ttf_context.load_font("assets/font.ttf", 128)?;
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    while parsed_intent != INTENT_DONE {
        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(&as_string(parsed_duration)[..]) // As a string slice
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

        parsed_duration = change_duration(parsed_duration, &parsed_intent);
    }   

    Ok(parsed_duration)
}


/// Hijacks and SDL context and displays a chronometer for the given amount of time provided
pub fn display_chronometer<T: crate::main_loop::sdl2::render::RenderTarget, U>( res_width : u32, res_height : u32,
                                                                                canvas : &mut Canvas<T>, 
                                                                                ttf_context : &sdl2::ttf::Sdl2TtfContext, 
                                                                                texture_creator : &TextureCreator<U>, 
                                                                                time : Duration ) -> Result<(), String> {
    let mut remaining = time;
    let limit = Duration::new(0,0);

    loop { 
        remaining = remaining.saturating_sub(Duration::from_millis(100));
        
        if remaining == limit { // Stop the chronometer
            break;
        };

        // Load the font
        let mut font = ttf_context.load_font("assets/font.ttf", 128)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        // render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(&as_string(remaining)[..]) // As a string slice
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

        // Sleep for a short time, since we don't want to the poor raspberry to catch in flames
        ::std::thread::sleep(Duration::from_millis(100));
    } 

    // Display and fire up the alarm sound
    canvas.clear();        
    let image = Path::new(ALARM_FACE);
    let texture = texture_creator.load_texture(image)?;
    canvas.copy(&texture, None, None)?;
    canvas.present();

    play_sound_blocking(ALARM_SOUND);
   
    Ok(())
}


// Return a Duration as a hh:mm:ss String
fn as_string(dur : Duration) -> String {
    let minutes = dur.as_secs() / 60;
    let seconds = dur.as_secs() % 60;

    let hours = minutes / 60;
    let minutes = minutes % 60;
    
    // right-aligned argument with a padding of 0's (09,08...)
    format!("{:0>2}:{:0>2}:{:0>2}",hours,minutes,seconds).to_owned() 
}


// Return the provided duration modified by the amount dictated by an intent
fn change_duration(dur : Duration, intent : &str) -> Duration {
    let dur_temp = match intent {   
        INTENT_5_MORE  => dur.saturating_add(Duration::from_secs(60*5)),
        INTENT_10_MORE => dur.saturating_add(Duration::from_secs(60*10)),
        INTENT_20_MORE => dur.saturating_add(Duration::from_secs(60*20)),
        INTENT_5_LESS  => dur.saturating_sub(Duration::from_secs(60*5)),
        INTENT_10_LESS => dur.saturating_sub(Duration::from_secs(60*10)),
        INTENT_20_LESS => dur.saturating_sub(Duration::from_secs(60*20)),
        _ => dur, // No matching intent found
    };

    dur_temp
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
