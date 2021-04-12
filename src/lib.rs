extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//better panics
extern crate console_error_panic_hook;
use std::panic;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct Universe {
    map: Vec<Room> 
}

#[derive(Clone, Debug)]
pub struct Room {
    desc: String,
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let mut state = Universe{
            map: Vec::new(),
        };
    
        state.map.push(Room{desc:"This is a pub. There's a counter along the furthest wall, and an assortment of tables and chairs.".to_string()});

        log!("We have a universe");

        // We'll return the state with the short-hand
        state
    }

    pub fn get_map(&self) -> String {
        let desc = &self.map[0].desc;
        //TODO: prettify output!
        return format!("{}", desc);
    }

}

pub fn main() {
    log!("We have a game!");
}


// Auto-starts on page load
//start section of the executable may not literally point to main
#[wasm_bindgen(start)]
pub fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    //main()
} 
