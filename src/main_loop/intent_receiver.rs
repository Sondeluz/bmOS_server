use std::net::{TcpStream, TcpListener};
use std::io::{BufReader, BufRead};
use std::sync::{Mutex, Arc, Condvar};

use super::StateMutex;

/// Listen and handle one connection on the given address and port. Whenever 
/// an intent is received, the current_intent and new_intent variables are updated.
pub fn listen(addr : String, port : String, state : StateMutex, new_intent_available : Arc<(Mutex<bool>, Condvar)>) -> std::io::Result<()> {
    
    let listener = TcpListener::bind(format!("{}:{}",addr, port))?;

    for stream in listener.incoming().take(1) { // Only handle one connection
        handle_client(stream?, Arc::clone(&state), Arc::clone(&new_intent_available))?;
    }

    Ok(())
}

/// Helper function for listen. 
fn handle_client(stream : TcpStream, state : StateMutex, new_intent_available : Arc<(Mutex<bool>, Condvar)>) -> std::io::Result<()> {
    let mut buffer = "".to_owned();
    
    let reader = BufReader::new(stream);

    for l in reader.lines() { // For every line that arrives from the client (its messages end in '\n')
        buffer.pop(); // Remove the delimiter

        if let Ok(mut state) = state.lock() {
            state.current_intent = l.unwrap().clone();
            // Signal that there is a new intent available
            state.new_intent = true;
        }

        let (lock, cvar) = &*new_intent_available;
       
        // Also let know auxiliary functions that there is a new intent
        if let Ok(mut new_intent) = lock.lock() {
            *new_intent = true;
            cvar.notify_one();
        }

        buffer.clear();
    }
    
    // Either the client finished the connection or errored out, either way we need to exit
    std::process::exit(1);
}
