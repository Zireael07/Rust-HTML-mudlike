use super::log;
use super::{Universe, Room, Exit, Distance, DataMaster, ItemPrefab,
    Player, GameMessages, Needs, Attributes, Attribute,
    AI, CombatStats, NPCName, EncDistance,
    Item, InBackpack, WantsToDropItem, WantsToUseItem, ToRemove, 
    Consumable, ProvidesHealing, ProvidesFood, ProvidesQuench,
    Equippable, Equipped, EquipmentSlot, MeleeBonus, DefenseBonus, Ranged,
    ScriptCommand, GLOBAL_SCRIPT_OUTPUT, register_var, DATA
};
use super::language;
use super::lispy;

use hecs::{Entity, EntityBuilder};

use std::collections::HashMap;

//RNG
use rand::Rng;

//Methods not exposed to JS
//https://stackoverflow.com/a/62575771
impl From<&ItemPrefab> for hecs::EntityBuilder {
    fn from(prefab: &ItemPrefab) -> hecs::EntityBuilder {
        let mut builder = EntityBuilder::new();
        builder.add(prefab.name.to_string());
        builder.add(prefab.item.unwrap());
        builder.add(prefab.equippable.unwrap());
        builder.add(prefab.defense.unwrap());

        return builder;
    }
}

impl Universe {
    //moved because of //https://github.com/rustwasm/wasm-bindgen/issues/111 preventing using our data
    pub fn game_start(&mut self, data: DataMaster) {
        //write the global data
        DATA.write().unwrap().items = data.items;

        //NOTE: trying to use &data throws weird errors here
        //load sentences into the generator
        for s in data.toki_pona {
            self.language.parse(&s, 2, false);
        }

        //load sentences into the generator
        for s in data.toki_pona_q {
            self.language.parse(&s, 2, true);
        }

        //do the rest of the setup
        language::setup(&mut self.language);

        //debug
        for (key, value) in &self.language.map {
            log!("{}: {:?}", key, value)
        }

        //load names to be scripting-accessible
        self.names = data.names;

        //setup the rooms
        for r in data.rooms {
            self.map.push(r);
        }

        // OBSOLETE: map now generated via scripting

        //now clone some capsules and place them in the hotel
        //let mut ca = self.map[5].clone();
        //self.map.push(ca);
        //let mut hallway = &mut self.map[4];
        //hallway.exits.push((Exit::West, 6));
        //Rust ranges are exclusive at the end!
        // for i in 0..2 {
        //     let mut ca = self.map[5].clone();
        //     ca.exits = vec![(Exit::West, 4)];
        //     self.map.push(ca);

        //     let mut hallway = &mut self.map[4];
        //     hallway.exits.push((Exit::East, 6+i+1));
        //     //log!("Created {} capsules", i);
        // }

        //log!("Hallway: {:?}", state.map[4].exits);

        //proceduralize the town
        let mut end = self.map.len();
        log!("End is {}", end);

        //store the end globally so that it can be accessed from both Rust and script
        register_var("end".to_string(), lispy::RispExp::Number(self.map.len() as f64));



        //test
        self.env.data.insert(
            "end".to_string(),
            lispy::RispExp::Number(self.map.len() as f64)

            //it's a closure due to using self... :(
            // lispy::RispExp::Func(
            //     |args: &[lispy::RispExp]| -> Result<lispy::RispExp, lispy::RispErr> {  
            //       Ok(lispy::RispExp::Number(self.map.len() as f64))
            //     }
            //   )
        
        );


        //debug 2
        log!("Risp env!");
        for (key, value) in &self.env.data {
            log!("{}:{}", key, value)
        }


        let mut rng = rand::thread_rng();
        let more : bool = rand::random(); //generates a boolean
        let max = rng.gen_range(1,3);
        //expose to scripting
        self.env.data.insert("more".to_string(), lispy::RispExp::Number((more as i32) as f64));
        log!("More: {} {}", more, ((more as i32) as f64));
        self.env.data.insert("map_max".to_string(), lispy::RispExp::Number(max as f64));
        log!("Max: {}", max);

        //finish up the starting line
        // for i in 0..1+more as i32 {
        //     let mut all = self.map[3].clone();
        //     let mut hov = self.map[2].clone();
        //     let mut stre = self.map[1].clone();
        //     all.exits = vec![(Exit::South, 1)];
        //     self.map.push(all);
        //     self.map.push(hov);
        // }

        // //log!("Hov is {:?}", &self.map[end+1]);
        // //log!("All is {:?}", &self.map[end]);

        // let mut street = &mut self.map[1];
        // street.exits.push((Exit::North, end));
        // street.exits.push((Exit::In, end+1));

        // end = self.map.len();

        // match max {
        //     1 => {
        //         for i in 0..1+more as i32 {
        //             let mut stre = self.map[1].clone();
        //             let mut hov = self.map[13].clone();
        //             let mut all = self.map[3].clone();
        //             let mut field = self.map[6].clone();
        //             //end+1 is stre below
        //             stre.exits = vec![(Exit::West, end+3), (Exit::South, 3), (Exit::North, end+2), (Exit::In, end+1)];
        //             hov.exits = vec![(Exit::Out, end)];
        //             all.exits = vec![(Exit::South, end)];
        //             field.exits = vec![(Exit::East, end), (Exit::West, 11)];
        //             self.map.push(stre);
        //             self.map.push(hov);
        //             self.map.push(all);
        //             self.map.push(field);
        //             let mut tower = &mut self.map[11];
        //             tower.exits.push((Exit::East, end+3));
        //         }
        //         let mut alley = &mut self.map[3];
        //         alley.exits.push((Exit::North, end));
        //         log!("Added northern street");
        //     },
        //     2 => {
        //         for i in 0..1+more as i32 {
        //             let mut stre = self.map[1].clone();
        //             let mut hov = self.map[18].clone();
        //             let mut all = self.map[3].clone();
        //             //end+1 is stre below
        //             stre.exits = vec![(Exit::North, 1), (Exit::South, end+2), (Exit::In, end+1)];
        //             hov.exits = vec![(Exit::Out, end)];
        //             all.exits = vec![(Exit::North, end)];
        //             self.map.push(stre);
        //             self.map.push(hov);
        //             self.map.push(all);
        //         }
        //         let mut street = &mut self.map[1];
        //         street.exits.push((Exit::South, end));
        //         log!("Added southern street")
        //     },
        //     3 => {
        //         //add to both north and south end
        //         for i in 0..1+more as i32 {
        //             let mut stre = self.map[1].clone();
        //             let mut hov = self.map[2].clone();
        //             let mut all = self.map[3].clone();
        //             let mut field = self.map[6].clone();
        //             //end+1 is stre below
        //             stre.exits = vec![(Exit::West, end+3), (Exit::South, 3), (Exit::North, end+2), (Exit::In, end+1)];
        //             hov.exits = vec![(Exit::Out, end)];
        //             all.exits = vec![(Exit::South, end)];
        //             field.exits = vec![(Exit::East, end), (Exit::West, 11)];
        //             self.map.push(stre);
        //             self.map.push(hov);
        //             self.map.push(all);
        //             self.map.push(field);
        //             let mut tower = &mut self.map[11];
        //             tower.exits.push((Exit::East, end+3));
        //         }
        //         let mut alley = &mut self.map[3];
        //         alley.exits.push((Exit::North, end));
        //         //south half
        //         let endi = self.map.len();
                
        //         for i in 0..1+more as i32 {
        //             let mut stre = self.map[1].clone();
        //             let mut hov = self.map[2].clone();
        //             let mut all = self.map[3].clone();
        //             //end+1 is stre below
        //             stre.exits = vec![(Exit::North, 1), (Exit::South, endi+2), (Exit::In, endi+1)];
        //             hov.exits = vec![(Exit::Out, endi)];
        //             all.exits = vec![(Exit::North, endi)];
        //             self.map.push(stre);
        //             self.map.push(hov);
        //             self.map.push(all);
        //         }
        //         let mut street = &mut self.map[1];
        //         street.exits.push((Exit::South, endi));
        //         log!("Added southern and nothern streets");
        //     },
        //     _ => {},
        // }

        //player dummy entity
        // 15, 14, 13, 12, 10, 8 aka elite array
        self.ecs_world.spawn(("Player".to_string(), Player{}, 0 as usize, CombatStats{hp:20, max_hp: 20, defense:1, power:1},  Needs{hunger:500, thirst:300},
         GameMessages{entries:vec![]},
         Attributes{strength:Attribute{base:2, bonus:0}, dexterity:Attribute{base:1, bonus:0}, constitution:Attribute{base:2, bonus:0}, intelligence:Attribute{base:1,bonus:0}, wisdom:Attribute{base:-1,bonus:0}, charisma:Attribute{base:0,bonus:0}}
        ));
        //starting inventory
        self.give_item("Protein shake".to_string());
        self.give_item("Medkit".to_string());

        //two parts of data aren't in a special struct - entity name and room it is in

        //test scripting
        log!("Test scripting...");
        lispy::read_eval(data.lisp_script, &mut self.env);
        //process all of the queued up commands from the lispy script here
        // using directly avoids weird borrow checker problems
        //let mut vec = &mut *GLOBAL_SCRIPT_OUTPUT.lock().unwrap();
        for cmd in &mut *GLOBAL_SCRIPT_OUTPUT.lock().unwrap() {
            match cmd.clone().unwrap() {
                ScriptCommand::GoRoom{id} => {
                    let new_room = id;
                    self.current_room = new_room;
                    //mark as known
                    self.know_room(new_room);
                    //log!("{}", &format!("New room {}", self.current_room));
                    self.end_turn();
                },
                ScriptCommand::Spawn{room, name} => {
                    let sp = self.ecs_world.spawn((name.trim().to_string(), room as usize));

                    //we can't match Strings :(
                    match name.as_str() {
                        "Thug" => {
                            self.ecs_world.insert(sp, (CombatStats{hp:10, max_hp:10, defense:1, power:1}, EncDistance{dist: Distance::Near}));
                            //let l_jacket = self.ecs_world.spawn((DATA.read().unwrap().items[1].name.to_string(), DATA.read().unwrap().items[1].item.unwrap(), DATA.read().unwrap().items[1].equippable.unwrap(), DATA.read().unwrap().items[1].defense.unwrap())); //ToRemove{yes:false}
                            let l_jacket = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[1]).build());
                            let boots = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[0]).build());
                            let jeans = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[2]).build());
                            self.ecs_world.insert_one(l_jacket, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Torso});
                            self.ecs_world.insert_one(boots, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Feet});
                            self.ecs_world.insert_one(jeans, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Legs});
                        },
                        "Shooter" => {
                            self.ecs_world.insert(sp, (CombatStats{hp:10, max_hp:10, defense:1, power:1}, EncDistance{dist: Distance::Far}));
                            let l_jacket = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[1]).build());
                            let boots = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[0]).build());
                            let jeans = self.ecs_world.spawn(EntityBuilder::from(&DATA.read().unwrap().items[2]).build());
                            self.ecs_world.insert_one(l_jacket, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Torso});
                            self.ecs_world.insert_one(boots, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Feet});
                            self.ecs_world.insert_one(jeans, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Legs});
                            let pistol = self.ecs_world.spawn((Item{}, Equippable{ slot: EquipmentSlot::Melee }, Ranged{}, ToRemove{yes:false}));
                            self.ecs_world.insert_one(pistol, Equipped{ owner: sp.to_bits(), slot: EquipmentSlot::Melee});
                        },
                        _ => {
                            //randomized NPC name
                            let sel_name = Universe::randomized_NPC_name(true, &self.names);
                            let nm = self.ecs_world.insert_one(sp, NPCName{name: sel_name.to_string()});
                            log!("{}", &format!("{}", sel_name.to_string()));
                        }
                    }

                },
                ScriptCommand::SpawnItem{room, name} => {
                    let sp = self.ecs_world.spawn((name.trim().to_string(), room as usize, Item{}));
                    match name.as_str() {
                        "Medkit" => { self.ecs_world.insert(sp, (Consumable{}, ProvidesHealing{heal_amount:5}, ToRemove{yes:false})); },
                        "Combat knife" => { self.ecs_world.insert(sp, (Equippable{ slot: EquipmentSlot::Melee }, MeleeBonus{ bonus: 2}, ToRemove{yes:false})); },
                        "Baton" => { self.ecs_world.insert(sp, (Equippable{ slot: EquipmentSlot::Melee }, MeleeBonus{ bonus: 1 }, ToRemove{yes:false})); },
                        "Pistol" => { self.ecs_world.insert(sp, (Equippable{ slot: EquipmentSlot::Melee }, ToRemove{yes:false})); },
                        "Leather jacket" => { self.ecs_world.insert(sp, (DATA.read().unwrap().items[1].item.unwrap(), DATA.read().unwrap().items[1].equippable.unwrap(), DATA.read().unwrap().items[1].defense.unwrap())); }, //ToRemove{yes:false}
                        "Camo" => { self.ecs_world.insert(sp, (DATA.read().unwrap().items[3].item.unwrap(), DATA.read().unwrap().items[3].equippable.unwrap(), DATA.read().unwrap().items[3].defense.unwrap())); },
                        "Boots" => { self.ecs_world.insert(sp, (DATA.read().unwrap().items[0].item.unwrap(), DATA.read().unwrap().items[0].equippable.unwrap(), DATA.read().unwrap().items[0].defense.unwrap())); },
                        "Jeans" => { self.ecs_world.insert(sp, (DATA.read().unwrap().items[2].item.unwrap(), DATA.read().unwrap().items[2].equippable.unwrap(), DATA.read().unwrap().items[2].defense.unwrap())); },
                        _ => { }
                    }
                },
                ScriptCommand::SpawnRoom{id} => {
                    //log!("Prev len: {}", self.map.len());
                    let r = self.map[id].clone();
                    log!("Spawned a room id {:?} ", r);
                    //add to the map
                    self.map.push(r);
                    log!("Map len: {}", self.map.len());
                },
                ScriptCommand::SetExit{id, exit, exit_to} => {
                    if id >= self.map.len() {
                        log!("Invalid id passed to set exit: {} ", id);
                        continue
                    }
                    let mut r = &mut self.map[id];
                    r.exits = vec![(Exit::from_u8(exit), exit_to)];
                    log!("Set exits for room {} - {:?}", id, r.exits);
                },
                ScriptCommand::AppendExit{id, exit, exit_to} => {
                    if id >= self.map.len() {
                        log!("Invalid id passed to set exit: {} ", id);
                        continue
                    }
                    let mut r = &mut self.map[id];
                    r.exits.push((Exit::from_u8(exit), exit_to));
                    log!("Exits for room {} - {:?}", id, r.exits);
                },
                ScriptCommand::RemoveExit{id, exit} => {
                    if id >= self.map.len() {
                        log!("Invalid id passed to set exit: {} ", id);
                        continue
                    }
                    let mut r = &mut self.map[id];
                    r.exits.retain(|&e| e.0 != Exit::from_u8(exit));
                }
                ScriptCommand::EditExit{id, exit, exit_to} => {
                    if id >= self.map.len() {
                        log!("Invalid id passed to set exit: {} ", id);
                        continue
                    }
                    let mut r = &mut self.map[id];
                    
                    //find our exit
                    // find() returns a reference, so we can't edit it later
                    let find = r.exits.iter().position(|e| e.0 == Exit::from_u8(exit));
                    log!("Find: {:?}", find);
                    
                    match find {
                        Some(i) => {r.exits[i].1 = exit_to},
                        None => log!("No exit found: {:?} ", Exit::from_u8(exit)),
                    }

                    //log!("Found {}", find);

                    log!("Exits for room {} - {:?}", id, r.exits);
                },
                _ => { log!("{}", format!("Unimplemented scripting command {:?} ", cmd)); }
            }
            

        }
        //unlock the mutex
        //drop(vec);
        //test (wtf is this not dropped?!)
        //log!("{:?}", vec);

        GLOBAL_SCRIPT_OUTPUT.lock().unwrap().clear();

    }

    pub fn get_entities_in_room(&self, rid: usize) -> Vec<u64> {
        let mut list = Vec::new();
        for (id, (room_id)) in self.ecs_world.query::<(&usize)>()
        .without::<InBackpack>().without::<Player>()
        .with::<String>()
        .iter() {
            if *room_id == rid {
                list.push(id.to_bits())
            }
        }
        return list;
    }

     //we store a list of ids and get the actual data with this separate function
    pub fn get_data_for_id(&self, id: u64) -> (u64, String, Option<Item>, Distance) {
        let ent = hecs::Entity::from_bits(id); //restore

        let name = self.ecs_world.get::<String>(ent).unwrap().to_string();
        let mut item: Option<Item> = None;

        if self.ecs_world.get::<Item>(ent).is_ok() {
            //need to dereference it
            item = Some(*self.ecs_world.get::<Item>(ent).unwrap())
        }

        //default to distance of medium
        let mut dist = Distance::Medium;

        if self.ecs_world.get::<EncDistance>(ent).is_ok() {
            dist = self.ecs_world.get::<EncDistance>(ent).unwrap().dist;
        }
        
        return (id, name, item, dist);
        
        //return format!("{} {}", id, name);
    }

    pub fn items_in_inventory(&self) -> Vec<Vec<(u64, String, Option<Equipped>)>>{
        let mut data = Vec::new();
        let mut tmp = Vec::new();
        //test
        for (id, (item, backpack)) in &mut self.ecs_world.query::<(&Item, &InBackpack)>().iter(){
            //log!("{}", &format!("Item in inventory: {}", self.ecs_world.get::<&str>(id).unwrap().to_string()));
            //log!("{}", &format!("ID: {:?}", id));
            //ids.push(id.to_bits());
            let name = self.ecs_world.get::<String>(id).unwrap().to_string();
            let mut equipped: Option<Equipped> = None;
            if self.ecs_world.get::<Equipped>(id).is_ok(){
                equipped = Some(*self.ecs_world.get::<Equipped>(id).unwrap());
            }
            tmp.push((id.to_bits(), name, equipped));
        }

        //has to be outside loop
        //split into two lists, one for equipped and the other for not
        let (equip,inv):(_,Vec<_>)=tmp
            .into_iter()
            .partition(|x| x.2.is_some());

        data = vec![equip, inv];

        return data;
    }

    pub fn get_player(&self) -> Option<Entity> {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        return play;
    }

    pub fn get_messages(&self) -> Vec<String> {
        let mut msg: Vec<String> = vec![];
        let player = self.get_player();
        if player.is_some(){
            //cannot move out of dereference
            //msg = self.ecs_world.get::<GameMessages>(player.unwrap()).unwrap().entries;
            //hack fix
            for e in self.ecs_world.get::<GameMessages>(player.unwrap()).unwrap().entries.iter() {
                msg.push(e.to_string());
            }
        }
        return msg;
    }

    pub fn get_stats(&self) -> String {
        let player = self.get_player();
        if player.is_some() {
            let stats = self.ecs_world.get::<Attributes>(player.unwrap()).unwrap();
            return format!("Player stats: {:?}", *stats);
        }
        return "".to_string();
    }

    pub fn give_item(&mut self, name: String) {
        //let current_room = self.current_room;

        let mut item: Option<Entity> = None;
        //TODO: should be a dict lookup
        if name == "Protein shake".to_string() {
            item = Some(self.ecs_world.spawn(("Protein shake".to_string(), self.current_room, Item{}, ProvidesFood{}, ProvidesQuench{}, Consumable{}, ToRemove{yes:false})));
        }
        if name == "Medkit".to_string() {
            item = Some(self.ecs_world.spawn(("Medkit".to_string(), self.current_room, Item{}, ToRemove{yes:false}, Consumable{}, ProvidesHealing{heal_amount:5})));
        }
        match item {
            Some(it) => {
                //puts the item in backpack
                self.pickup_item(&it);
            },
            None => {},
        }

    }


    pub fn pickup_item(&mut self, item: &Entity) {
        self.ecs_world.insert_one(*item, InBackpack{});
        //self.items_in_inventory();
    }

    pub fn drop_item(&mut self, user: &Entity, it: &Entity) {
        // The indirection is here to make it possible for non-player Entities to drop items
        //tell the engine that we want to drop the item
        self.ecs_world.insert_one(*user, WantsToDropItem{item:*it});

        //scope to get around borrow checker
        {
            for (id, (wantstodrop)) in self.ecs_world.query::<(&WantsToDropItem)>().iter(){
                let mut room = self.ecs_world.get_mut::<usize>(wantstodrop.item).unwrap();
                *room = self.current_room;
            }
        }

        self.ecs_world.remove_one::<InBackpack>(*it);
        
    }

    pub fn use_item(&mut self, user: &Entity, it: &Entity) {
        // The indirection is here to make it possible for non-player Entities to use items
        //tell the engine that we want to use the item
        self.ecs_world.insert_one(*user, WantsToUseItem{item:*it});

        //message
        log!("{}", &format!("{} used {}", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<String>(*it).unwrap().to_string()));
        // apply the use effects
        let mut wants : Vec<Entity> = Vec::new();
        let mut to_unequip : Vec<Entity> = Vec::new();
        for (id, (wantstouse)) in self.ecs_world.query::<(&WantsToUseItem)>().iter(){
            //log!("{}", &format!("Want to use item: {:?}", wantstouse.item));
            //log!("{}", &format!("Item: {}", self.ecs_world.get::<String>(wantstouse.item).unwrap().to_string()));

            // If it heals, apply the healing
            // NOTE: no & here!!!
            if self.ecs_world.get::<ProvidesHealing>(wantstouse.item).is_ok() {
                //actually heal!
                let mut stats = self.ecs_world.get_mut::<CombatStats>(*user).unwrap();
                stats.hp += self.ecs_world.get::<ProvidesHealing>(wantstouse.item).unwrap().heal_amount;
                let player = self.get_player();
                let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();
                log.entries.push(format!("{} heals {} damage", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<ProvidesHealing>(wantstouse.item).unwrap().heal_amount));                
            } else {
                log!("Item doesn't provide healing");
            }

            // // food or drink?
            if self.ecs_world.get::<ProvidesQuench>(wantstouse.item).is_ok(){
                let player = self.get_player();
                let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap(); 
                log.entries.push(format!("You drink the {}", self.ecs_world.get::<String>(*it).unwrap().to_string()));
            } else if self.ecs_world.get::<ProvidesFood>(wantstouse.item).is_ok(){
                let player = self.get_player();
                let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap(); 
                log.entries.push(format!("You eat the {}", self.ecs_world.get::<String>(*it).unwrap().to_string()));
            }

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            if self.ecs_world.get::<Equippable>(wantstouse.item).is_ok() {
                //if it's equipped already...
                if self.ecs_world.get::<Equipped>(wantstouse.item).is_ok(){
                    let equipped = self.ecs_world.get::<Equipped>(wantstouse.item).unwrap();
                    let owner = hecs::Entity::from_bits(equipped.owner);
                    if owner == *user {
                        to_unequip.push(wantstouse.item);
                        //if target == *player_entity {
                        let player = self.get_player();
                        let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();    
                        log.entries.push(format!("You unequip {}.", self.ecs_world.get::<String>(wantstouse.item).unwrap().to_string()));
                    }
                }
                else {
                    let can_equip = self.ecs_world.get::<Equippable>(wantstouse.item).unwrap();
                    let target_slot = can_equip.slot;
            
                    // Remove any items the target has in the item's slot
                    //let mut to_unequip : Vec<Entity> = Vec::new();
    
                    //find items in slot
                    for (ent_id, (equipped)) in self.ecs_world.query::<(&Equipped)>()
                    .with::<String>()
                    .iter()
                    {
                        let owner = hecs::Entity::from_bits(equipped.owner);
                        if owner == *user && equipped.slot == target_slot {
                            to_unequip.push(ent_id);
                            //if target == *player_entity {
                            let player = self.get_player();
                            let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();    
                            log.entries.push(format!("You unequip {}.", self.ecs_world.get::<String>(ent_id).unwrap().to_string()));
                        }   
                    }
                    wants.push(wantstouse.item);
                    let player = self.get_player();
                    let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();
                    log.entries.push(format!("{} equips {}", self.ecs_world.get::<String>(*user).unwrap().to_string(), self.ecs_world.get::<String>(*it).unwrap().to_string()));
                }
               
            }

            if self.ecs_world.get::<Consumable>(wantstouse.item).is_ok() {
                log!("Item is a consumable");
                //FIXME: we can't add components or remove entities while iterating, so this is a hack
                self.ecs_world.get_mut::<ToRemove>(wantstouse.item).unwrap().yes = true;
            }
        }

        // deferred some actions because we can't add or remove components when iterating
        for item in to_unequip.iter() {
            self.ecs_world.remove_one::<Equipped>(*item);
        }

        for item in wants.iter() {
            let eq = { //scope to get around borrow checker
                let get = self.ecs_world.get::<Equippable>(*item).unwrap();
                *get //clone here to get around borrow checker
            };
            // slot related to item's slot
            self.ecs_world.insert_one(*item, Equipped{owner:user.to_bits(), slot:eq.slot});
            
            //self.ecs_world.remove_one::<InBackpack>(*item);
        }

    }

    //test: a number of positive + negative tokens, select one
    fn make_test(&self, skill: u32) -> bool {
        let mut toks = Vec::new();
        for i in 1..=skill {
            //first three tokens are positive (think of it like rolling under 3 or over 17 on a i-sized die)
            if i <= 3 {
                toks.push(1); //positive tokens
            }
            else {
                toks.push(-1); //negative tokens
            }
        }

        let mut rng = rand::thread_rng();
        let sel = rng.gen_range(0, toks.len());

        if toks[sel] == 1 {
            return true; //succeeded
        } else {
            return false;
        }

    }

    //a very simple test, akin to flipping a coin or throwing a d2
    fn make_test_d2(&self, skill: u32) -> Vec<bool> {
        let mut rolls = Vec::new();
        for _ in 0..10-skill { // exclusive of end
            rolls.push(rand::random()) // generates a boolean
        }
        return rolls
    }

    pub fn attack(&self, target: &Entity) {
        //let res = self.make_test_d2(1);
        //let sum = res.iter().filter(|&&b| b).count(); //iter returns references and filter works with references too - double indirection
        //game_message(&format!("Test: {} sum: {{g{}", Rolls(res), sum));

        let res = self.make_test(6);

        if res {
        //if sum >= 5 {
            //game_message(&format!("Attack hits!"));

            //item bonuses
            let mut offensive_bonus = 0;
            for (id, (power_bonus, equipped_by)) in self.ecs_world.query::<(&MeleeBonus, &Equipped)>().iter() {
                //if equipped_by.owner == attacker {
                    offensive_bonus += power_bonus.bonus;
            }

            //deal damage
            // the mut here is obligatory!!!
            let mut stats = self.ecs_world.get_mut::<CombatStats>(*target).unwrap();
            stats.hp = stats.hp - 2 - offensive_bonus;
            
            let player = self.get_player();
            let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();
            log.entries.push(format!("Dealt {} damage", 2+offensive_bonus));
            
            //can't remove dead here due to borrow checker
        } else {
            let player = self.get_player();
            let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();
            log.entries.push(format!("Attack missed!"));
        }
    }

    pub fn end_turn(&mut self) {
        self.get_AI();
        self.remove_dead();
        self.survival_tick();
    }

    fn get_AI(&mut self) {
        for (id, (ai, room_id)) in &mut self.ecs_world.query::<(&AI, &usize)>()
        .with::<String>()
        .iter()
        {
            if *room_id == self.current_room {
                //get player entity
                let mut play: Option<Entity> = None;
                for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
                    play = Some(id);
                }
                match play {
                    Some(entity) => {
                        self.attack(&entity);
                        let mut log = self.ecs_world.get_mut::<GameMessages>(entity).unwrap();
                        log.entries.push(format!("AI {} kicked at the player", self.ecs_world.get::<String>(id).unwrap().to_string()));
                    },
                    None => {},
                }
            }
            
        }
    }

    fn remove_dead(&mut self) {
        // Here we query entities with 0 or less hp and despawn them
        let mut to_remove: Vec<Entity> = Vec::new();
        let mut to_drop : Vec<(Entity)> = Vec::new();
        for (id, stats) in &mut self.ecs_world.query::<&CombatStats>() {
            if stats.hp <= 0 {
                if id.id() > 0 { 
                    to_remove.push(id);
                }
                // player - just a log message
                else {
                    let player = self.get_player();
                    let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();
                    log.entries.push(format!("You are DEAD!"));
                }
            }
        }

        for (id, remove) in &mut self.ecs_world.query::<&ToRemove>() {
            if remove.yes {
                to_remove.push(id);
            }
        }

        for entity in to_remove {
            // not item
            if self.ecs_world.get::<Item>(entity).is_err() {
                //drop their stuff
                for (ent_id, (equipped)) in self.ecs_world.query::<(&Equipped)>()
                .with::<String>()
                .iter()
                {
                    let owner = hecs::Entity::from_bits(equipped.owner);
                    if owner == entity {
                        to_drop.push((ent_id));
                    }
                }

                //log message
                let player = self.get_player();
                let mut log = self.ecs_world.get_mut::<GameMessages>(player.unwrap()).unwrap();
                log.entries.push(format!("AI {} is dead", self.ecs_world.get::<String>(entity).unwrap().to_string()));
            }
            
            self.ecs_world.despawn(entity).unwrap();
        }

        // deferred some actions because we can't add or remove components when iterating
        for it in to_drop.iter() {
            self.ecs_world.remove_one::<Equipped>(*it);
            //put in current room
            self.ecs_world.insert_one(*it, self.current_room);

            log!("{}", &format!("Dropping item {} room {} ", self.ecs_world.get::<String>(*it).unwrap().to_string(), self.current_room));
        }

    }

    fn survival_tick(&mut self) {
        //get player entity
        let mut play: Option<Entity> = None;
        for (id, (player)) in self.ecs_world.query::<(&Player)>().iter() {
            play = Some(id);
        }
        match play {
            Some(entity) => {
                let mut needs = self.ecs_world.get_mut::<Needs>(entity).unwrap();
                needs.hunger -= 1;
                needs.thirst -= 1;
            },
            None => {},
        }
    }

    pub fn randomized_NPC_name(male: bool, names: &HashMap<String, Vec<String>>) -> String {
        let mut rng = rand::thread_rng();

        //we know the key exists
        let SPANISH_MALE = &names["spanish_male"];
        let SPANISH_FEMALE = &names["spanish_female"];
        let SPANISH_LAST = &names["spanish_last"];
        //log!("{}", &format!("female names {:?} ", names["spanish_female"]));

        if !male {
            let sel_id = rng.gen_range(0, SPANISH_FEMALE.len());
            let sel_name = &SPANISH_FEMALE[sel_id];
    
            let sell_id = rng.gen_range(0, SPANISH_LAST.len());
            let last_name = &SPANISH_LAST[sell_id];
            return format!("{} {}", sel_name, last_name);            
        } else {
            let sel_id = rng.gen_range(0, SPANISH_MALE.len());
            let sel_name = &SPANISH_MALE[sel_id];
            let sell_id = rng.gen_range(0, SPANISH_LAST.len());
            let last_name = &SPANISH_LAST[sell_id];
            return format!("{} {}", sel_name, last_name);
        }
    }

    // -------------
    pub fn print_components(&self, entity: Entity) -> String {
        let mut desc = "".to_string();
        if self.ecs_world.get::<CombatStats>(entity).is_ok() {
            let tmp = format!("{:?}", *self.ecs_world.get::<CombatStats>(entity).unwrap());
            desc = format!("{} {}", desc, tmp);
        }
        if self.ecs_world.get::<Item>(entity).is_ok(){
            let tmp = format!("{:?}", *self.ecs_world.get::<Item>(entity).unwrap());
            desc = format!("{} {}", desc, tmp);
        }
        if self.ecs_world.get::<DefenseBonus>(entity).is_ok(){
            let tmp = format!("{:?}", *self.ecs_world.get::<DefenseBonus>(entity).unwrap());
            desc = format!("{} {}", desc, tmp);
        }
        if self.ecs_world.get::<ProvidesHealing>(entity).is_ok(){
            let tmp = format!("{:?}", *self.ecs_world.get::<ProvidesHealing>(entity).unwrap());
            desc = format!("{} {}", desc, tmp);
        }

        if self.ecs_world.get::<Equippable>(entity).is_ok() {
            let tmp = format!("{:?}", *self.ecs_world.get::<Equippable>(entity).unwrap());
            desc = format!("{} {}", desc, tmp);
        }

        if self.ecs_world.get::<Equipped>(entity).is_ok() {
            let tmp = format!("{:?}", *self.ecs_world.get::<Equipped>(entity).unwrap());
            desc = format!("{} {}", desc, tmp);
        }

        return desc
    }


    pub fn debug_console_core(&mut self, input:String) {
        //split by spaces
        //let v: Vec<&str> = input.split(' ').collect();
        //debug
        log!("{}", &format!("{:?}", input));

        lispy::read_eval(input, &mut self.env);
        self.process("".to_string()); //dummy
    }

} // end of Universe impl