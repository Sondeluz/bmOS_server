//! # bmOS_server
//!
//! bmOS_server is an executable in charge of receiving intents and rendering their associated BMO-faces and 
//! playing audio tracks.
//!
//! ## Configuration files
//! The following configuration files are required to be present in the same folder the executable is in:
//! - **faces.txt** : Indicates the image files of BMO's faces to be shown for each intent. It's mandatory to have at least one entry for each intent which will be sent from the client, except for the weather and chronometer functionalities preset intents. Otherwise, the application will panic.
//! - **audio.txt** : Indicates the audio tracks to be played for each intent. It can be empty
//! - **timings.txt** : Indicates the time limits for each intent. It is mandatory to have one entry for each intent without an audio track, again excluding the preset intents.
//! 
//! **Information about the syntax and contents needed in each configuration files is present in the documentation of the functions inside the config module.**
//!
//! ## Mandatory intents
//! The following intents are mandatory to have faces defined in faces.txt:
//! - **"default"**: In order to show BMO's default/fallback face.
//!
//! The following files are mandatory to be present in the executables folder:
//! - **./assets/faces/alarm.jpg** : Alarm face to be shown after a chronometer finishes.
//! - **./assets/audio/alarm.wav** : Alarm audio track to be played after a chronometer finishes.
//! - **./assets/font.ttf** : Font to be used when showing text. I recommend [Video Terminal Screen](https://ttfonts.net/en/download/62485.htm)
//! 
//! ## Shutdown
//! - bmOS_server listens for key inputs. If the escape key is pressed, the application will exit.
//! - If it's running in a headless server, the appropiate way of exiting is to close bmOS_client (or any other source) which is sending intents to it. This will trigger a safe shutdown.
//! ## Assumptions
//! The following assumptions are made when running this application:
//! - openAL, SDL2 and SDL2-ttf libraries are installed in the system
//! - If the device running the bmOS_server is the audio (microphone) source, it's streaming it to the device running bmOS_client by some other means. bmOS_server does not record any audio, and only listens to strings received to its provided address.
//!
//! ## Recommendations
//! - Since the paths for the mandatory files are pre-determined, I advice to have all assets in an "assets" folder wherever the executable is in.
//! - Provide appropiate timings for each intent. For example, setting 100 for a given intent without an audio track will make it zoom past it and go back to the default state. A good timing that I found for such intents is 4500 (4'5 seconds).



use std::env;

use std::error::Error;

mod config;
pub use crate::config::{parse_assets, parse_timings};

mod main_loop;
pub use crate::main_loop::run;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        println!("Incorrect arguments.\n
                    Usage: ./server own_address own_port resolution_width resolution_height [OpenWeather_API_KEY] [Location (city...)] [Country code]\n
                    Example: ./server 192.168.1.15 2300 800 600 f07[...]b42 Zaragoza ES");
        std::process::exit(-1);
    }

    let intent_faces = config::parse_assets("faces.txt").unwrap();
    let intent_audio = config::parse_assets("audio.txt").unwrap();
    let intent_timings = config::parse_timings("timings.txt").unwrap();

    println!("Asset locations parsed successfully, starting...");

    let res_width = args[3].clone().parse::<u32>().unwrap();
    let res_height = args[4].clone().parse::<u32>().unwrap();

    if args.len() == 6 {
        main_loop::run(args[1].clone(), args[2].clone(), res_width, res_height, Some(args[5].clone()), Some(args[6].clone()), Some(args[7].clone()), intent_faces, intent_audio, intent_timings).unwrap();
    } else {
        main_loop::run(args[1].clone(), args[2].clone(), res_width, res_height, None, None, None, intent_faces, intent_audio, intent_timings).unwrap();
    }

    

    Ok(())
}
