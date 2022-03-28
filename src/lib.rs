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

//lazy static
#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;

// is trick to be able to store variables :P
use std::sync::{Arc, RwLock};

//tin
use std::convert::TryInto;

//ECS
use hecs::World;
use hecs::{Entity, EntityBuilder};

//RNG
use rand::Rng;

mod universe_private;
use universe_private::*;
mod lispy;
use lispy::{RispExp, RispErr, parse_list_of_floats, parse_single_float};
mod language;
use language::{Markov};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub base : i32, // equal to what would've been the modifier in d20
    pub bonus : i32
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Attributes {
    pub strength : Attribute,
    pub dexterity : Attribute,
    pub constitution : Attribute,
    pub intelligence : Attribute,
    pub wisdom : Attribute,
    pub charisma : Attribute,
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
#[derive(Copy, Clone, Serialize, Deserialize)] //temporarily while I figure out something better
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
pub struct Ranged {}

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
    names: HashMap<String, Vec<String>>,
    language: Markov,
    env: lispy::RispEnv,
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
    pub melee: Option<MeleeBonus>,
    pub consumable: Option<Consumable>,
    pub heal: Option<ProvidesHealing>,
    pub ranged: Option<Ranged>,
    pub rem: Option<ToRemove>, //temporarily while I figure out what to do
}

//what it says
#[derive(Deserialize)]
pub struct DataMaster {
    pub rooms : Vec<Room>,
    pub names : HashMap<String, Vec<String>>,
    pub items: Vec<ItemPrefab>,
    pub toki_pona: Vec<String>,
    pub toki_pona_q: Vec<String>,
    pub lisp_script: String,
}

pub struct DataGlobal {
    pub items: Vec<ItemPrefab>,
    pub item_index: HashMap<String, usize>,
    pub room_index: HashMap<String, usize>,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Distance {
    Near = 0,
    Medium = 1,
    Far = 2
}


//default of 25 C = 298 K
struct Weather {
    pub temp: f32, //in Kelvin, to avoid negatives
}

lazy_static! {
    static ref GLOBAL_SCRIPT_OUTPUT: Mutex<Vec<Option<ScriptCommand>>> = Mutex::new(vec![]);
    //based on https://github.com/WesterWest/calisp/commit/b6a18038cfed2c02aa3f9b3228b07e6846200277
    static ref GLOBAL_SCRIPT_VARIABLES: Arc<RwLock<HashMap<String, RispExp>>> =
    Arc::new(RwLock::new(HashMap::default()));
    //need it to be global to be able to access it from script
    //RWLock allows one writer and multiple readers, therefore avoiding a mutex deadlock in scripting processing
    static ref DATA: Arc<RwLock<DataGlobal>> = Arc::new(RwLock::new(DataGlobal::new()));
}
//pub static mut GLOBAL_SCRIPT_OUTPUT: Option<ScriptCommand> = None;

//https://dev.to/mnivoliez/getting-started-with-rust-enum-on-steroids-8b4
// turns out Rust enums can contain more data...
#[derive(Debug, PartialEq, Clone)]
pub enum ScriptCommand {
    GoRoom { id: usize },
    Spawn { room: usize, name:String },
    SpawnItem { room: usize, name:String },
    SpawnRoom { name: String },
    SetExit { id: usize, exit: u8, exit_to: usize },
    AppendExit { id: usize, exit: u8, exit_to: usize },
    RemoveExit { id: usize, exit: u8 },
    EditExit { id: usize, exit: u8, exit_to: usize },
    DebugList,
    DebugEntity { id: usize },
}

//https://enodev.fr/posts/rusticity-convert-an-integer-to-an-enum.html
//to convert from number back to enum
impl Exit {
    fn from_u8(value: u8) -> Exit {
        match value {
            0 => Exit::North,
            1 => Exit::East,
            2 => Exit::South,
            3 => Exit::West,
            4 => Exit::Out,
            5 => Exit::In,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

//the part of DataMaster that can be safely opened to all of the game
// DataMaster itself doesn't need a new() since it's built from RON
impl DataGlobal {
    pub fn new() -> DataGlobal {
        DataGlobal {
            items: Vec::new(),
            item_index : HashMap::new(),
            room_index: HashMap::new(),
        }
    }
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

    // for s in &data.toki_pona {
    //     log!("{}", s);
    // }
    
    //game_start() deals with everything else we may need data for (including making part of it global)
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
            language: Markov::new(),
            names: HashMap::new(),
            env: lispy::RispEnv::new(),
        };

        //state.map.push(Room{name:"Pub".to_string(), desc:"This is a pub. There's a counter along the furthest wall, and an assortment of tables and chairs.".to_string(), known: true, exits: vec![(Exit::Out, 1)]});

        state.env = lispy::default_env();
        state.env.data.insert(
            "go".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;
                let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
                log!("go {}", format!("{}", first));

                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::GoRoom{id: first as usize}));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "spawn".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let float = parse_single_float(args.get(0).unwrap())?; //ok_or(RispErr::Reason("expected a number".to_string())))?;

                let second = args.get(1).ok_or(
                    RispErr::Reason(
                      "expected second form".to_string(),
                    )
                  )?;
                log!("{}", format!("spawning {} {}", float, second));

                //I don't know a better way to do it, this avoids having to use state
                // this monster strips quote characters from around the lispy string
                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::Spawn{room: float as usize, name: second.to_string().strip_suffix("\"").unwrap().strip_prefix("\"").unwrap().to_string() }));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "spawn_item".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let float = parse_single_float(args.get(0).unwrap())?; //ok_or(RispErr::Reason("expected a number".to_string())))?;

                let second = args.get(1).ok_or(
                    RispErr::Reason(
                      "expected second form".to_string(),
                    )
                  )?;
                log!("{}", format!("spawning {} {}", float, second));

                //I don't know a better way to do it, this avoids having to use state
                // this monster strips quote characters from around the lispy string
                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::SpawnItem{room: float as usize, name: second.to_string().strip_suffix("\"").unwrap().strip_prefix("\"").unwrap().to_string() }));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "spawn_room".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                //let floats = parse_list_of_floats(args)?;
                //let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
                let first = args.get(0).ok_or(
                    RispErr::Reason(
                      "expected first form".to_string(),
                    )
                  )?;
                

                //do all necessary replacements/stripping
                let key = &first.to_string().strip_suffix("\"").unwrap_or(&format!("Error {}", &first.to_string())).strip_prefix("\"").unwrap_or(&format!("Error {}", first.to_string())).to_string().replace("_", " ");
                
                //log!("spawning room id {}", id);

                let mut num = 0;
                
                //MAGIC! we're automatically getting the end variable without going through state!
                let var = get_var("end".to_string());
                let nm = match var.unwrap() {
                    RispExp::Number(s) => Ok(s.clone()),
                    _ => Err(RispErr::Reason(
                        "expected end to be a number".to_string(),
                      ))
                    }?; 
                
                num = nm as usize;

                //NOTE: assumes we can always add a room!
                //automatically calculate how many rooms we have
                let new_end = num + 1; 
                // reflect this in the end variable
                register_var("end".to_string(), RispExp::Number(new_end as f64));

                // using state causes this to become a closure :(
                //state.env.data.insert("end".to_string(), num_added);

                //this is the scripting command
                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::SpawnRoom{name: key.to_string()}));

                log!("Returning num: {}", new_end-1);
                //Ok(RispExp::Bool(true))
                Ok(RispExp::Number((new_end-1) as f64)) //return the number of the room added (one less than the end)
              }
            )
          );
          state.env.data.insert(
            "set_exit".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;

                log!("Setting exit for id {} - {} to {}", floats[0] as usize, floats[1] as u8, floats[2] as usize);
                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::SetExit{id: floats[0] as usize, exit: floats[1] as u8, exit_to: floats[2] as usize}));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "append_exit".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;
                log!("Appended exit for id {} - {} to {}", floats[0] as usize, floats[1] as u8, floats[2] as usize);

                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::AppendExit{id: floats[0] as usize, exit: floats[1] as u8, exit_to: floats[2] as usize}));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "replace_exit".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;
                log!("Edit exit for id {} - {} to {}", floats[0] as usize, floats[1] as u8, floats[2] as usize);

                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::EditExit{id: floats[0] as usize, exit: floats[1] as u8, exit_to: floats[2] as usize}));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "remove_exit".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;
                log!("Remove exit for id {} {}", floats[0] as usize, floats[1] as u8);

                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::RemoveExit{id: floats[0] as usize, exit: floats[1] as u8}));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "debug_list".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                //let floats = parse_list_of_floats(args)?;
                //let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
                log!("debug");

                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::DebugList));

                Ok(RispExp::Bool(true))
              }
            )
          );
          state.env.data.insert(
            "debug".to_string(), 
            RispExp::Func(
              |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;
                let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
                log!("debug {}", format!("{}", first));

                GLOBAL_SCRIPT_OUTPUT.lock().unwrap().push(Some(ScriptCommand::DebugEntity{id: floats[0] as usize}));

                Ok(RispExp::Bool(true))
              }
            )
          );

        //lispy::slurp_eval(&mut state.env);

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

    pub fn examine(&self) -> JsValue {
        let stats = self.get_stats();
        return JsValue::from_str(&stats);
    }

    fn know_room(&mut self, id: usize) {
        self.map[id].known = true;
    }

    pub fn get_known_exits(&self) -> JsValue {
        let room = &self.map[self.current_room];
        let mut known_ex = Vec::new();
        for e in &room.exits {
            if self.map[e.1].known {
                known_ex.push(&self.map[e.1]);
            }
        }
        return JsValue::from_serde(&known_ex).unwrap();
    }

    pub fn get_more_known(&self) -> JsValue {
        let mut known_ex = Vec::new();
        
        //get all known exits
        let room = &self.map[self.current_room];
        let mut rooms = Vec::new();
        for e in &room.exits {
            if self.map[e.1].known {
                rooms.push(&self.map[e.1]);
            }
        }

        for (idx, r) in rooms.iter().enumerate() {
            for e in &r.exits {
                //if exit known and not current room
                if self.map[e.1].known && e.1 != self.current_room {
                    known_ex.push((idx, &self.map[e.1]));
                    break; //we only need the first one for each room
                }
            }
        }
        return JsValue::from_serde(&known_ex).unwrap();
    }


    pub fn get_sentences(&mut self, question: bool) -> JsValue {
        let mut rng = rand::thread_rng();
        //let max = rng.gen_range(2,3);
        let max_s = rng.gen_range(3,4);
        let mut sentences = String::new();
        match question {
            true => { sentences = self.language.display_sentence(question, "".to_string()); },
            false => { 
                sentences = self.language.display_paragraph(max_s); 

                //NaNoGenMo output
                //log!("{}", self.language.display_novel());
                
                //sentences = self.language.display_block(max, max_s);
                //let words = sentences.split(" ").collect::<Vec<&str>>();
                //let word_count = words.len();
                //log!("Sentences length: {}", word_count);
                // a single block has from 30 to 50 words
            }
        }


        return JsValue::from_serde(&sentences).unwrap();
    }

    pub fn get_tokens(&self, s: String) -> JsValue {
        let tokens = language::tokenize(s);
        return JsValue::from_serde(&tokens).unwrap();
    }

    // what it says on the tin
    // it handles in-game commands, NOT script!
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
                    let player = self.get_player();
                    if player.is_some(){
                        self.attack(&player.unwrap(), &entity);                    
                        //enemy turn
                        self.end_turn();
                    }
                }
                //language is handled in get_sentences() above
            },
                
            _ => { log!("Unknown command entered"); }
        }
    }

    pub fn console_input(&mut self, input:String) {
        log!("Rust console input: {}", input);
        
        self.debug_console_core(input);
    }

    // main game process
    pub fn process(&mut self, cmd: String) {
        log!("Rust engine input cmd: {}", cmd);

        self.command_handler(cmd);

        //handle script engine
        unsafe {

            //don't use let here because it makes problems, see universe_private l.350...
            //let vec = &*GLOBAL_SCRIPT_OUTPUT.lock().unwrap();
            for cmd in &mut *GLOBAL_SCRIPT_OUTPUT.lock().unwrap() {
                //debug
                log!("cmd: {:?}", cmd);
                //TODO: deduplicate/reuse code for script commands processing from universe_private.rs (l.235)
                match cmd.clone().unwrap() {
                    ScriptCommand::GoRoom{id} => {
                        let new_room = id;
                        self.current_room = new_room;
                        //mark as known
                        self.know_room(new_room);
                        //log!("{}", &format!("New room {}", self.current_room));
                        self.end_turn();
                    },
                    ScriptCommand::DebugList => {
                        for (id, (room_id)) in &mut self.ecs_world.query::<&usize>() {
                            log!("{:?} {:?}", id.to_bits(), format_entity(self.ecs_world.entity(id).unwrap()));
                        }
                    }
                    ScriptCommand::DebugEntity{id} => {
                        let entity = hecs::Entity::from_bits(id.try_into().unwrap()); //restore
                        log!("{:?} {:?} {:?}", entity, format_entity(self.ecs_world.entity(entity).unwrap()), self.print_components(entity));
                    },
                    ScriptCommand::Spawn{room, name} => {
                        let sp = self.ecs_world.spawn((name.trim().to_string(), room as usize));
                        //we can't match Strings :(
                        match name.as_str() {
                            //made DATA global for the sole reason of being able to use it here

                            "Thug" => {
                                self.ecs_world.insert(sp, (AI{}, CombatStats{hp:10, max_hp:10, defense:1, power:1}, EncDistance{dist: Distance::Near}));
                                let l_jacket = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[DATA.read().unwrap().item_index["Leather jacket"]]).build());
                                let boots = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[DATA.read().unwrap().item_index["Boots"]]).build());
                                let jeans = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[DATA.read().unwrap().item_index["Jeans"]]).build());
                                self.ecs_world.insert_one(l_jacket, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Torso});
                                self.ecs_world.insert_one(boots, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Feet});
                                self.ecs_world.insert_one(jeans, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Legs});
                            },
                            "Shooter" => {
                                self.ecs_world.insert(sp, (AI{}, CombatStats{hp:10, max_hp:10, defense:1, power:1}, EncDistance{dist: Distance::Far}));
                                let l_jacket = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[DATA.read().unwrap().item_index["Leather jacket"]]).build());
                                let boots = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[DATA.read().unwrap().item_index["Boots"]]).build());
                                let jeans = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[DATA.read().unwrap().item_index["Jeans"]]).build());
                                self.ecs_world.insert_one(l_jacket, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Torso});
                                self.ecs_world.insert_one(boots, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Feet});
                                self.ecs_world.insert_one(jeans, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Legs});
                                let pistol = self.ecs_world.spawn((EntityBuilder::from(&DATA.read().unwrap().items[DATA.read().unwrap().item_index["Pistol"]]).build()));
                                self.ecs_world.insert_one(pistol, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Melee});
                            },
                            _ => {
                                //randomized NPC name
                                let sel_name = Universe::randomized_NPC_name(true, &self.names);
                                let nm = self.ecs_world.insert_one(sp, NPCName{name: sel_name.to_string()});
                                log!("{}", &format!("{}", sel_name.to_string()));
                            }
                        }
                    }
                    _ => { log!("{}", format!("Unimplemented scripting command {:?} ", cmd)); }
                } //match ends

            }
            GLOBAL_SCRIPT_OUTPUT.lock().unwrap().clear();
        }

        //clear
        // unsafe {
        //     if GLOBAL_SCRIPT_OUTPUT.lock().unwrap().len() == 1 {
        //         GLOBAL_SCRIPT_OUTPUT.lock().unwrap()[0] = None;
        //     }
            
        // }
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

//based on https://github.com/WesterWest/calisp/commit/b6a18038cfed2c02aa3f9b3228b07e6846200277
// Register rust variable that can be set from within script
pub fn register_var(v_name: String, v: RispExp) {
    GLOBAL_SCRIPT_VARIABLES.write().unwrap().insert(v_name, v);
}

pub fn get_var(v_name:String) -> Result<RispExp, RispErr> {
    match GLOBAL_SCRIPT_VARIABLES.read().unwrap().get(&v_name.clone()) {
        Some(variable) => Ok(variable.clone()),
        _ => Err(RispErr::Reason("rust-var not found".to_string())),    
    }
}

    //hecs examples/format.rs
    pub fn format_entity(entity: hecs::EntityRef<'_>) -> String {
        fn fmt<T: hecs::Component + std::fmt::Display>(entity: hecs::EntityRef<'_>) -> Option<String> {
            Some(entity.get::<T>()?.to_string())
        }
    
        const FUNCTIONS: &[&dyn Fn(hecs::EntityRef<'_>) -> Option<String>] =
            //types we want printed (so far, just those which are guaranteed to exist)
            &[&fmt::<String>, &fmt::<usize>];
    
        let mut out = String::new();

        for f in FUNCTIONS {
            if let Some(x) = f(entity) {
                if out.is_empty() {
                    out.push_str("[");
                } else {
                    out.push_str(", ");
                }
                out.push_str(&x);
            }
        }
        if out.is_empty() {
            out.push_str(&"[]");
        } else {
            out.push(']');
        }
        out
    }