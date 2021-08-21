extern crate sdl2;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path}; 

/// Parse assets from a specified filename into a HashMap.
/// # Format
/// Each intent name needs to be enclosed between [...], and have
/// each full path to the asset to be loaded in a new line below it.
/// The paths are not checked, so incorrect ones will result in a panic 
/// upon using them.
/// Empty lines are ignored.
///
/// This function is used for faces and audio tracks.
///
/// Audio track entries for any given intent can be excluded. This
/// will result in the face associated to it to have a specified
/// time limit.
///
/// # Example file
/// [hello]
///
/// /home/whoever/bmOS_server/assets/audio/hello/1.wav
///
/// /home/whoever/bmOS_server/assets/audio/hello/2.wav
/// 
///
/// [song]
///
/// ...
///
/// # Result
/// The HashMap will have an entry for each intent read, with
/// a vector of read paths associated to it.
/// 
/// The paths provided will be used randomly whenever a new intent
/// is read.
///
/// # Panic
/// Each intent needs to have, at the very least, one path entry, or else
/// it will result in a panic.
pub fn parse_assets(filename : &str) -> Result<HashMap<String, Vec<String>>, std::io::Error> {
    let mut intents : HashMap<String, Vec<String>> = HashMap::new(); 
    let mut files : Vec<String> = Vec::new();

    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut entry = String::new();

    for line in reader.lines() {
        let line = line.unwrap();

        if ! line.is_empty() { // ignore blank lines
            if line.starts_with("[") { // new intent
                if ! entry.is_empty() { intents.insert(entry, files.clone()); } // insert the previous intent's vector into the map (if it's not the first one)
                files.clear(); // and clear the files vector

                entry = line.replace("[", "").replace("]", "").trim().to_owned();
                //println!("new entry: '{}'", entry);
            } else { // new entry for the current intent
                assert_ne!(entry, "", "Tried to parse a file entry without an associated intent ([intent_name]...) above.");
                
                files.push(line);
            }
        }
    }

    intents.insert(entry.clone(), files); // insert the last intent's vector into the map

    Ok(intents)
}

/// Parse time limits from a specified filename into a HashMap.
/// # Format
/// Each intent name needs to be enclosed between [...], and have
/// the time limit in a new line below it, in milliseconds, 
/// with no decimal values.
/// Empty lines are ignored. Multiple time limit lines will result
/// in only the latest one being taken into account.
///
/// This function is used exclusively for reading the time limits.
/// 
/// If an intent has any audio track, the time limit will be ignored,
/// so it's not needed to add it. Otherwise, it's mandatory to have one
/// entry.
///
/// # Example file
/// [hello]
///
/// 200
/// 
/// [song]
///
/// 3500
///
/// # Result
/// The HashMap will have an entry for each intent read, with
/// a time limit associated to it.
/// 
/// # Panic
/// Each intent needs to have, at the very least, one time limit entry, or else
/// it will result in a panic.
pub fn parse_timings(filename : &str) -> Result<HashMap<String, u64>, std::io::Error> {
    let mut intents : HashMap<String, u64> = HashMap::new(); 

    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut entry = String::new();
    let mut val = 0;

    for line in reader.lines() {
        let line = line.unwrap();

        if ! line.is_empty() { // ignore blank lines
            if line.starts_with("[") { // new intent
                entry = line.replace("[", "").replace("]", "").trim().to_owned();
                //println!("new entry: '{}'", entry);
            } else { // new entry for the current intent
                assert_ne!(entry, "", "Tried to parse a file entry without an associated intent ([intent_name]...) above.");
                val = line.parse::<u64>().expect("Couldn't parse one of the timings, please ensure that it's a valid number (200, 3400...)");

                intents.insert(entry.clone(), val);
            }
        }
    }

    intents.insert(entry.clone(), val); // insert the last intent's vector into the map

    Ok(intents)
}


/// Load previously parsed assets from a specified HashMap (the result of parse_assets) into a HashMap of intents and lists of in-memory files
pub fn load_assets(assets : &HashMap<String, Vec<String>>) -> Result<HashMap<String, Vec<Vec<u8>>>, std::io::Error> {
    let mut result : HashMap<String, Vec<Vec<u8>>> = HashMap::new(); 

    for tuple in assets.iter() {
        let intent = tuple.0;
        
        let mut files : Vec<Vec<u8>> = Vec::new();

        for path in tuple.1.iter() {
            let path = Path::new(path);
            let file = fs::read(path)?;
            files.push(file);
        }
    
        result.insert(intent.clone(), files);
    }

    Ok(result)
}
