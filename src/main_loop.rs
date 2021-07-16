extern crate sdl2;
extern crate random_number; // For loading random assets for a given intent

// SDL libs
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;

use std::path::{Path}; // for providing paths to SDL
use std::time::Duration; // Sleeping
use std::collections::HashMap;

// Threads and synchronization for audio and communications with the server
use std::sync::{Arc, Mutex, Condvar}; 
use std::thread;

// Wrapper around rodio
mod audio_player;
pub use crate::main_loop::audio_player::{play_sound, play_sound_blocking};

// Communications with the server, where we will receive new intents
mod intent_receiver;
pub use crate::main_loop::intent_receiver::listen;

// Chronometer functions
mod chronometer;
pub use crate::main_loop::chronometer::{get_time, display_chronometer};

// Weather function
mod weather;
use crate::main_loop::weather::show_weather;

/// State of BMO's current face and audio track
pub struct State {
    pub current_intent : String, // Current intent, updated on listen()
    pub audio_finished : bool, // Did the audio track (if played) finish already?, updated on play_sound()
    pub new_intent : bool // Is there a new intent available?, updated on listen()
}

impl State {
    pub fn new() -> State {
        State { current_intent : "default".to_owned(), audio_finished : false, new_intent : false }
    }
}

// Thread-safe State instance
pub type StateMutex = Arc<Mutex<State>>;

const CHRONOMETER_STATE : &str = "chronometer";
const WEATHER_STATE : &str = "weather";
//const RES_WIDTH : u32 = 320;
//const RES_HEIGHT : u32 = 240;

/// Main loop for the SDL "game". New intents received are parsed on each
/// iteration, otherwise falling back to the "default" intent.
///
/// Faces and audio tracks for the current intent are chosen randomly from
/// the corresponding asset vectors.
///
/// The intent will display the same random image until either its associated
/// audio track (if any) stops playing. If it doesn't have any audio track, the
/// time limit will dictate how many milliseconds will the intent stay before changing.
///
/// The main loop ends upong pressing Escape.
pub fn run( address : String, port : String, 
            res_width : u32, res_height : u32,
            api_key : Option<String>, location : Option<String>, 
            country : Option<String>, 
            intent_faces : HashMap<String, Vec<String>>, 
            intent_audio : HashMap<String, Vec<String>>,
            intent_timings : HashMap<String, u64> ) -> Result<(), String> {
    // SDL initialization
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG)?;
    let window = video_subsystem
        .window("rust-sdl2 demo: Video", res_width, res_height)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();    

    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let state : StateMutex = Arc::new(Mutex::new(State::new()));
   
    let new_intent_available = Arc::new((Mutex::new(false), Condvar::new())); // Condvar for auxiliary functions (chronometer...)
    
    let new_intent_available_clone = Arc::clone(&new_intent_available);
    // The thread closure captures the parameters, so we need to declare them cloned beforehand, and then move them inside
    let state_clone = Arc::clone(&state);

    thread::spawn(move || {intent_receiver::listen(address, port, state_clone, new_intent_available_clone).unwrap(); });

    // Status variables
    let sleep_time = 100; // milliseconds between each iteration
    
    let mut time_limit = 0; // Time limit for the intent, dictated on "timings.txt"
    let mut time_slept = u64::MAX; // Counts how many milliseconds BMO has slept so far (I really hope nobody tells it to sleep 18446744073709551615 milliseconds)
    let mut audio_available = false; // Is there an audio track for the current intent?
    let mut played_audio = false; // Has the audio track, if present, been played?
    let mut loaded_face = false; // Has the intent's face been presented?
    let mut time_limit_loaded = false;

    let mut current_face = ""; // Current face's path, it will get loaded on the first iteration
    let mut current_intent_clone = "default".to_owned(); // In order to prevent changing the intents mid-iteration, we keep a local copy

    'mainloop: loop {       
        if let Ok(mut state) = state.lock() { // Lock the state struct
            // If there is audio available and it has been played already, 
            // or there was no audio and the time limit has been reached, 
            // get a new intent (or switch to the default one)
            if audio_available && played_audio && state.audio_finished || 
                !audio_available && time_slept > time_limit {
                //println!("audio_available: {}, played_audio: {}, audio_finished: {}, time_slept: {},", audio_available, played_audio, *audio_finished.lock().unwrap(), time_slept);
                if ! state.new_intent {
                    current_intent_clone  = "default".to_owned();
                    //println!("---------------------------");
                    //println!("Changing to default intent");
                    //println!("---------------------------");
                } else {
                    if state.current_intent == CHRONOMETER_STATE || state.current_intent == WEATHER_STATE {
                        current_intent_clone = state.current_intent.to_owned(); 
                    } else {
                        // Switch to a new state. If it's a preset one or it doesn't exist, skip it.
                        match intent_faces.get(&state.current_intent) {
                            Some(_) => {
                                current_intent_clone = state.current_intent.to_owned(); 
                            } 
                            None => {current_intent_clone  = "default".to_owned();},
                        };
                    }

                    state.new_intent = false;

                    // Since we also captured the state here, set to false its availability
                    // to auxiliary functions
                    let (lock, _) = &*new_intent_available;
                    let mut n_i = lock.lock().unwrap();
                    *n_i = false;

                    println!("---------------------------");
                    println!("new intent!, changing to {}", current_intent_clone);
                    println!("---------------------------");
                }
            
                // Reset the local status variables
                time_slept = 0;
                audio_available = false;
                played_audio = false;
                loaded_face = false;
                time_limit_loaded = false;
                
                state.audio_finished = false;
            }
        }

        if current_intent_clone == CHRONOMETER_STATE { // Hijack the canvas and display a chronometer
            let duration = get_time(res_width, res_height, &mut canvas, &ttf_context, &texture_creator, Arc::clone(&state),            
                                    Arc::clone(&new_intent_available))?; 
            display_chronometer(res_width, res_height, &mut canvas, &ttf_context, &texture_creator, duration)?;
            current_intent_clone  = "default".to_owned(); // Switch to the default state, we have finished here
        } else if current_intent_clone == WEATHER_STATE { // Hijack the canvas and display a chronometer
            // If the optional parameters were provided
            if let Some(ref key) = api_key {
                if let Some(ref location) = location {
                    if let Some(ref country) = country {
                        show_weather(res_width, res_height, &key, &location, &country, &mut canvas, 
                                        &ttf_context, &texture_creator, Arc::clone(&state),             
                                        Arc::clone(&new_intent_available))?; 
                    }
                }
            } else {
                eprintln!("Asked for weather, but didn't provide enough arguments at launch: ignoring");
            }
            current_intent_clone  = "default".to_owned(); // Switch to the default state, we have finished here
        }


        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => { }
            }
        }

        canvas.clear();
        
        // Look for a face to load
        if ! loaded_face || audio_available { // If a face hasn't been loaded yet, OR it's an intent where BMO talks
            current_face = match intent_faces.get(&current_intent_clone) {
                Some(x) => &x[random_number::random!(0, x.len() - 1)],
                None => panic!("No faces found for intent {}.", current_intent_clone),
            };
            
            loaded_face = true;
        }

        if ! time_limit_loaded {
            // Look for the time limit
            time_limit = match intent_timings.get(&current_intent_clone) {
                Some(x) => *x,
                None => panic!("No timing found for intent {}.", current_intent_clone),
            };

            time_limit_loaded = true;
        }

        // Look for a sound to play
        if ! played_audio {
            if let Some(current_audio) = intent_audio.get(&current_intent_clone) {      
                let state_clone = Arc::clone(&state);
                play_sound(current_audio[random_number::random!(0, current_audio.len() - 1)].clone(), state_clone);
                
                played_audio = true;
                audio_available = true;
            } else {
                played_audio = true; // We "played" it, avoids searching in the HashMap again
                audio_available = false;
            }
        }

        // Update the canvas
        let image = Path::new(current_face);
        let texture = texture_creator.load_texture(image)?;
        canvas.copy(&texture, None, None)?;
        canvas.present();

        // Sleep for a short time, since we don't want to the poor raspberry to catch in flames
        ::std::thread::sleep(Duration::from_millis(sleep_time));
        time_slept += sleep_time;
    } 

    Ok(())
}
