use std::thread;
use std::sync::Arc;

use soloud::*;

use super::StateMutex;


/// Plays a sound asynchronously, and sets the pointed state's audio_finished to true 
/// once it finishes.
/// The behavior is undefined if multiple sounds are played at the same time with the
/// same state instance.
pub fn play_sound(path : String, state : StateMutex ) {
    let state_clone = Arc::clone(&state);

    thread::spawn(move || {
        let sl = Soloud::default().unwrap();

        let mut wav = audio::Wav::default();

        wav.load(&std::path::Path::new(&path)).unwrap();

        sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
        while sl.active_voice_count() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if let Ok(mut state) = state_clone.lock() {
            state.audio_finished = true; // Signal that the audio track is already finished
        }
    });
}


/// Plays a sound, blocking until it's finished.
pub fn play_sound_blocking(path : &str) {
    let sl = Soloud::default().unwrap();

    let mut wav = audio::Wav::default();

    wav.load(&std::path::Path::new(&path)).unwrap();

    sl.play(&wav); // calls to play are non-blocking, so we put the thread to sleep
    while sl.active_voice_count() > 0 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}


// Holds a thread handle and forces a wait-sleep to the caller if a sound is already being played
// Not currently in use anywhere
/*
pub struct UniqueAudioPlayer {
    handle: Option<thread::JoinHandle<()>>,
}

impl UniqueAudioPlayer {
    pub fn new() -> UniqueAudioPlayer {
        UniqueAudioPlayer {
            handle: None,
        }
    }

    pub fn play_sound(&mut self, path : &'static str) {
        match self.handle {
            Some(_) =>  { // Wait for the previous thread to finish
                            self.handle.take().unwrap().join().expect("Could not join spawned thread");
                            self.start_thread(path);
                        },    
            None => self.start_thread(path),
        }
    
        
    }

    fn start_thread(&mut self, path : &'static str) {
        self.handle = Some(thread::spawn(move || {                                    
                        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                        let sink = Sink::try_new(&stream_handle).unwrap();

                        // Load a sound from a file, using a path relative to Cargo.toml
                        let file = BufReader::new(File::open(path).unwrap());
                        // Decode that sound file into a source
                        let source = Decoder::new(file).unwrap();

                        sink.append(source);
    
                        // The sound plays in a separate thread. This call will block the current thread until the sink
                        // has finished playing all its queued sounds.
                        sink.sleep_until_end();
                    }))
    }
}
*/
