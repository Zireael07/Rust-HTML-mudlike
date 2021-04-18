extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//for fetching data files
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

//better panics
extern crate console_error_panic_hook;
use std::panic;

use serde::{Serialize, Deserialize};

mod universe_private;
use universe_private::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct Universe {
    map: Vec<Room>,
    current_room: usize, //TODO: will likely be a property of player
}


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Exit {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
    Out = 4,
    In = 5
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    desc: String,
    known: bool,
    name: String,
    exits: Vec<(Exit, usize)>, //vector indices seem to be usizes?
}

//what it says
#[derive(Deserialize)]
pub struct DataMaster {
    pub rooms : Vec<Room>,
}


/// Public methods, exported to JavaScript.
#[wasm_bindgen]
//async loader based on https://rustwasm.github.io/docs/wasm-bindgen/examples/fetch.html
// returning Universe as a workaround for https://github.com/rustwasm/wasm-bindgen/issues/1858
pub async fn load_datafile(mut state: Universe) -> Universe {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = "./data.ron";

    let request = Request::new_with_str_and_init(&url, &opts).unwrap(); //no ? because we don't return Result

    request
        .headers();
        //.set("Accept", "application/vnd.github.v3+json")?;
        //.unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap(); //?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`, and then to string
    let ron = JsFuture::from(resp.text().unwrap()).await.unwrap().as_string().unwrap(); //?;

    log!("Loaded from rust: {}", &format!("{:?}", ron));

    let data : DataMaster = ron::from_str(&ron).expect("malformed file");

    //debug
    for r in &data.rooms {
        log!("{}", &format!("From data: {} {} {} {:?}", r.name, r.desc, r.known, r.exits));
    }

    state.game_start(data);

    return state
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let mut state = Universe{
            map: Vec::new(),
            current_room: 0,
        };

        //state.map.push(Room{name:"Pub".to_string(), desc:"This is a pub. There's a counter along the furthest wall, and an assortment of tables and chairs.".to_string(), known: true, exits: vec![(Exit::Out, 1)]});

        log!("We have a universe");

        // We'll return the state with the short-hand
        state
    }

    // returns a JSON object
    // we could do it otherwise but this way we're gonna have a consistent API
    // https://rustwasm.github.io/docs/wasm-bindgen/reference/arbitrary-data-with-serde.html
    pub fn get_current_map(&self) -> JsValue {
        let room = &self.map[self.current_room];
        
        return JsValue::from_serde(room).unwrap();
        //return format!("{} \n Exits: {:?}", room.desc, room.exit.0);
    }

    // see comment above
    pub fn get_room_id(&self, id: usize) -> JsValue {
        let room = &self.map[id];
        return JsValue::from_serde(room).unwrap();
    }


    fn know_room(&mut self, id: usize) {
        self.map[id].known = true;
    }

    // what it says on the tin
    fn command_handler(&mut self, cmd: String) {
        //split by spaces
        let v: Vec<&str> = cmd.split(' ').collect();
        //debug
        log!("{}", &format!("{:?}", v));
        match v[0] {
            "go" => {
                let new_room = v[1].parse::<usize>().unwrap();
                self.current_room = new_room;
                //mark as known
                self.know_room(new_room);
                //log!("{}", &format!("New room {}", self.current_room));
            },
            _ => { log!("Unknown command entered"); }
        }
    }


    // main game process
    pub fn process(&mut self, cmd: String) {
        log!("Rust engine input cmd: {}", cmd);

        self.command_handler(cmd);
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
