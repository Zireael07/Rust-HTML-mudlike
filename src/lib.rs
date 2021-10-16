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
use std::collections::HashMap;

//ECS
use hecs::World;
use hecs::Entity;


mod universe_private;
use universe_private::*;
mod lispy;
use lispy::{RispExp, RispErr, parse_list_of_floats, parse_single_float};
mod language;
use language::{Markov, add_text};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

//ECS
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Player{}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Needs{
    pub hunger: i32,
    pub thirst: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AI {}

pub struct NPCName {
    pub name: String
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CombatStats {
    pub max_hp : i32,
    pub hp : i32,
    pub defense : i32,
    pub power : i32
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Item{}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InBackpack{}

//don't need to be serialized
pub struct WantsToUseItem {
    pub item : Entity
}
pub struct WantsToDropItem {
    pub item : Entity
}
// tells the engine to nuke us
pub struct ToRemove {pub yes: bool} //bool is temporary while we can't modify entities when iterating

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee, Torso, Legs, Feet }
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Equippable {
    pub slot : EquipmentSlot
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Equipped {
    pub owner : u64, //because Entity cannot be serialized by serde
    pub slot : EquipmentSlot
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Consumable{} //in the sense that it is limited use only
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProvidesFood {}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProvidesQuench {}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct MeleeBonus {
    pub bonus : i32
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DefenseBonus {
    pub bonus : f32
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EncDistance {
    pub dist: Distance
}

//game log
pub struct GameMessages {
    pub entries: Vec<String>
}

#[wasm_bindgen]
pub struct Universe {
    map: Vec<Room>,
    current_room: usize, //TODO: will likely be a property of player
    ecs_world: World,
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
    id: usize,
    desc: String,
    known: bool,
    name: String,
    exits: Vec<(Exit, usize)>, //vector indices seem to be usizes?
}

//for data loading
#[derive(Serialize, Deserialize)]
pub struct ItemPrefab {
    pub name: String,
    pub item: Option<Item>,
    pub equippable: Option<Equippable>,
    pub defense: Option<DefenseBonus>,
}

//what it says
#[derive(Deserialize)]
pub struct DataMaster {
    pub rooms : Vec<Room>,
    pub names : HashMap<String, Vec<String>>,
    pub items: Vec<ItemPrefab>,
}


#[repr(u8)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Distance {
    Near = 0,
    Medium = 1,
    Far = 2
}

struct Weather {
    pub temp: f32, //in Kelvin
}


pub static mut GLOBAL_SCRIPT_OUTPUT: Option<ScriptCommand> = None;

//https://dev.to/mnivoliez/getting-started-with-rust-enum-on-steroids-8b4
// turns out Rust enums can contain more data...
#[derive(Debug, PartialEq, Clone)]
pub enum ScriptCommand {
    GoRoom { id: usize },
    Spawn { room: usize, name:String},
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
        log!("{}", &format!("From data: {} {} {} {} {:?}", r.id, r.name, r.desc, r.known, r.exits));
    }

    // for n in &data.names {
    //     log!("{}", &format!("Loaded names: {:?}", n));
    // }

    for i in &data.items {
        log!("{}", &format!("{} {:?} {:?} {:?}", i.name, i.item, i.equippable, i.defense));
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
            ecs_world: World::new(),
        };

        //state.map.push(Room{name:"Pub".to_string(), desc:"This is a pub. There's a counter along the furthest wall, and an assortment of tables and chairs.".to_string(), known: true, exits: vec![(Exit::Out, 1)]});

        //test scripting
        log!("Test scripting...");
        let env = &mut lispy::default_env();
        env.data.insert(
            "go".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;
                let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
                log!("{}", format!("{}", first));

                //I don't know a better way to do it, this avoids having to use state
                // based on the non-textual version's commands, which was then based on bracketlib's input handling
                unsafe {
                    GLOBAL_SCRIPT_OUTPUT = Some(ScriptCommand::GoRoom{id: first as usize});
                }
                Ok(RispExp::Bool(true))
              }
            )
          );
          env.data.insert(
            "spawn".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let float = parse_single_float(args.get(0).unwrap())?; //ok_or(RispErr::Reason("expected a number".to_string())))?;
                //let first = *float.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
                let second = args.get(1).ok_or(
                    RispErr::Reason(
                      "expected second form".to_string(),
                    )
                  )?;
                log!("{}", format!("{} {}", float, second));

                //I don't know a better way to do it, this avoids having to use state
                // based on the non-textual version's commands, which was then based on bracketlib's input handling
                unsafe {
                    // this monster strips quote characters from around the lispy string
                    GLOBAL_SCRIPT_OUTPUT = Some(ScriptCommand::Spawn{room: float as usize, name: second.to_string().strip_suffix("\"").unwrap().strip_prefix("\"").unwrap().to_string() });
                }
                Ok(RispExp::Bool(true))
              }
            )
          );

        lispy::read_eval(env);

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

    pub fn display_entities_in_room(&self) -> JsValue {
        //let room = &self.map[self.current_room];
        let entities = self.get_entities_in_room(self.current_room);

        //separated because in the future, we'll want to do more interesting stuff probably
        let mut disp = Vec::new();
        for e in entities {
            disp.push(self.get_data_for_id(e));
        }

        //let inv = self.get_items_in_inventory();

        return JsValue::from_serde(&disp).unwrap();
    }

    pub fn display_inventory(&self) -> JsValue {
        let inv = self.items_in_inventory();

        return JsValue::from_serde(&inv).unwrap();
    }

    pub fn display_messages(&self) -> JsValue {
        let messages = self.get_messages();
        return JsValue::from_serde(&messages).unwrap();
    }

    fn know_room(&mut self, id: usize) {
        self.map[id].known = true;
    }

    pub fn get_sentences(&self) -> JsValue {
        let mut lang = Markov::new();
        add_text(&mut lang);
        //debug
        for (key, value) in &lang.map {
            log!("{}: {:?}", key, value)
        }

        //log!("{:?}", lang.generate_sentence());
        let sentences = lang.generate_sentence();
        return JsValue::from_serde(&sentences).unwrap();
    }

    pub fn get_tokens(&self, s: String) -> JsValue {
        let tokens = language::tokenize(s);
        return JsValue::from_serde(&tokens).unwrap();
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
                self.end_turn();
            },
            "get" => {
                let item_id = v[1].parse::<u64>().unwrap();
                let ent = hecs::Entity::from_bits(item_id); //restore
                self.pickup_item(&ent);
                //enemy turn
                self.end_turn();
            },
            "drop" => {
                let item_id = v[1].parse::<u64>().unwrap();
                let ent = hecs::Entity::from_bits(item_id); //restore
                let player = self.get_player();
                if player.is_some(){
                    //log!("Dropping item");
                    self.drop_item(&player.unwrap(), &ent);
                    //enemy turn
                    self.end_turn();
                }
            },
            "use" => {
                let item_id = v[1].parse::<u64>().unwrap();
                let ent = hecs::Entity::from_bits(item_id); //restore
                let player = self.get_player();
                if player.is_some(){
                    //log!("Dropping item");
                    self.use_item(&player.unwrap(), &ent);
                    //enemy turn
                    self.end_turn();
                }
            },
            "npc_interact" => {
                let id = v[1].parse::<u64>().unwrap();
                let entity = hecs::Entity::from_bits(id); //restore
                if self.ecs_world.get::<CombatStats>(entity).is_ok() {
                    log!("This is an enemy, attack");
                    self.attack(&entity);                    
                    //enemy turn
                    self.end_turn();
                }
                //language is handled in get_sentences() above

                //fn generate(num: i32) {
                // let mut lang = Markov::new();
                // add_text(&mut lang);
                // //debug
                // for (key, value) in &lang.map {
                //     log!("{}: {:?}", key, value)
                // }

                // log!("{:?}", lang.generate_sentence());
                //}
            },
                
            _ => { log!("Unknown command entered"); }
        }
    }


    // main game process
    pub fn process(&mut self, cmd: String) {
        log!("Rust engine input cmd: {}", cmd);

        self.command_handler(cmd);

        //handle script engine
        unsafe {
            if GLOBAL_SCRIPT_OUTPUT != None {
                log!("script output {:?}", GLOBAL_SCRIPT_OUTPUT);
                //unfortunately we need a clone here to work with non-Copy enum
                match GLOBAL_SCRIPT_OUTPUT.clone().unwrap() {
                    ScriptCommand::GoRoom{id} => {
                        let new_room = id;
                        self.current_room = new_room;
                        //mark as known
                        self.know_room(new_room);
                        //log!("{}", &format!("New room {}", self.current_room));
                        self.end_turn();
                    },
                    ScriptCommand::Spawn{room, name} => {
                        self.ecs_world.spawn((name.trim().to_string(), room as usize));
                    }
                    _ => { log!("{}", format!("Unimplemented scripting command {:?} ", GLOBAL_SCRIPT_OUTPUT)); }
                }
            }
        }

           

        //clear
        unsafe {
            GLOBAL_SCRIPT_OUTPUT = None;
        }
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
