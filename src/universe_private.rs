use super::log;
use super::{Universe, Room, Exit, DataMaster, 
    Player, CombatStats, Item, InBackpack, WantsToDropItem
};

use hecs::Entity;

//RNG
use rand::Rng;

//Methods not exposed to JS
impl Universe {
    //moved because of //https://github.com/rustwasm/wasm-bindgen/issues/111 preventing using our data
    pub fn game_start(&mut self, data: DataMaster) {
    
        for r in data.rooms {
            self.map.push(r);
        }

        // self.map.push(Room{name:"Pub".to_string(), desc:"This is a pub. There's a counter along the furthest wall, and an assortment of tables and chairs.".to_string(), known: true, exits: vec![(Exit::Out, 1)]});
        // self.map.push(Room{name:"Street".to_string(), desc:"The town street looks a bit deserted at this hour. The sky is overcast and it looks like it's going to rain any moment.".to_string(), known: false, exits: vec![(Exit::In,0), (Exit::North, 3), (Exit::In, 2)]});
        // self.map.push(Room{name:"Hovel".to_string(), desc:"This place looks like a dump. Dust and cobwebs rule the corners, but a part of the room is clearly lived in - there's a desk, a lamp, a simple stove and what looks like a bedroll.".to_string(), known: false, exits: vec![(Exit::Out, 1)]});
        // self.map.push(Room{name:"Alley".to_string(), desc:"This is a narrow, cramped alleyway lit by dim, flickering neon signs. Cables swing overhead to the tune of the whistling wind.".to_string(), known: false, exits: vec![(Exit::South, 1), (Exit::In, 4)]});
        // self.map.push(Room{name:"Hotel hallway".to_string(), desc:"This is a small hotel's hallway. Both walls are lined with identical doors, with green or red neon lights overhead.".to_string(), known: false, exits: vec![(Exit::Out, 3), (Exit::West, 5)]});
        
        //let mut cap = Room{name:"Capsule".to_string(), desc:"This is a tiny capsule, roughly a person's height across. At least enough so one can lay down comfortably and there's an overhead storage space for anything a person might have, too.".to_string(), known: false, exits: vec![(Exit::East, 4)]};
        //self.map.push(cap);

        //now clone some capsules and place them in the hotel
        let mut ca = self.map[5].clone();
        self.map.push(ca);
        let mut hallway = &mut self.map[4];
        hallway.exits.push((Exit::West, 6));
        //Rust ranges are exclusive at the end!
        for i in 0..2 {
            let mut ca = self.map[5].clone();
            ca.exits = vec![(Exit::West, 4)];
            self.map.push(ca);

            let mut hallway = &mut self.map[4];
            hallway.exits.push((Exit::East, 6+i+1));
            //log!("Created {} capsules", i);
        }

        //log!("Hallway: {:?}", state.map[4].exits);

        //proceduralize the town
        let mut end = self.map.len();
        log!("End is {}", end);
        let mut rng = rand::thread_rng();
        let more : bool = rand::random(); //generates a boolean
        let max = rng.gen_range(1,3);

        //finish up the starting line
        for i in 0..1+more as i32 {
            let mut all = self.map[3].clone();
            let mut hov = self.map[2].clone();
            let mut stre = self.map[1].clone();
            all.exits = vec![(Exit::South, 1)];
            self.map.push(all);
            self.map.push(hov);
        }

        //log!("Hov is {:?}", &self.map[end+1]);
        //log!("All is {:?}", &self.map[end]);

        let mut street = &mut self.map[1];
        street.exits.push((Exit::North, end));
        street.exits.push((Exit::In, end+1));

        end = self.map.len();

        match max {
            1 => {
                for i in 0..1+more as i32 {
                    let mut stre = self.map[1].clone();
                    let mut hov = self.map[2].clone();
                    let mut all = self.map[3].clone();
                    //end+1 is stre below
                    stre.exits = vec![(Exit::South, 3), (Exit::North, end+2), (Exit::In, end+1)];
                    hov.exits = vec![(Exit::Out, end)];
                    all.exits = vec![(Exit::South, end)];
                    self.map.push(stre);
                    self.map.push(hov);
                    self.map.push(all);
                }
                let mut alley = &mut self.map[3];
                alley.exits.push((Exit::North, end));

            },
            2 => {
                for i in 0..1+more as i32 {
                    let mut stre = self.map[1].clone();
                    let mut hov = self.map[2].clone();
                    let mut all = self.map[3].clone();
                    //end+1 is stre below
                    stre.exits = vec![(Exit::North, 1), (Exit::South, end+2), (Exit::In, end+1)];
                    hov.exits = vec![(Exit::Out, end)];
                    all.exits = vec![(Exit::North, end)];
                    self.map.push(stre);
                    self.map.push(hov);
                    self.map.push(all);
                }
                let mut street = &mut self.map[1];
                street.exits.push((Exit::South, end));
            },
            3 => {
                //add to both north and south end
                for i in 0..1+more as i32 {
                    let mut stre = self.map[1].clone();
                    let mut hov = self.map[2].clone();
                    let mut all = self.map[3].clone();
                    //end+1 is stre below
                    stre.exits = vec![(Exit::South, 3), (Exit::North, end+2), (Exit::In, end+1)];
                    hov.exits = vec![(Exit::Out, end)];
                    all.exits = vec![(Exit::South, end)];
                    self.map.push(stre);
                    self.map.push(hov);
                    self.map.push(all);
                }
                let mut alley = &mut self.map[3];
                alley.exits.push((Exit::North, end));
                //south half
                let endi = self.map.len();
                
                for i in 0..1+more as i32 {
                    let mut stre = self.map[1].clone();
                    let mut hov = self.map[2].clone();
                    let mut all = self.map[3].clone();
                    //end+1 is stre below
                    stre.exits = vec![(Exit::North, 1), (Exit::South, endi+2), (Exit::In, endi+1)];
                    hov.exits = vec![(Exit::Out, endi)];
                    all.exits = vec![(Exit::North, endi)];
                    self.map.push(stre);
                    self.map.push(hov);
                    self.map.push(all);
                }
                let mut street = &mut self.map[1];
                street.exits.push((Exit::South, endi));
            },
            _ => {},
        }

        //player dummy entity
        self.ecs_world.spawn((Player{}, 0 as usize, CombatStats{hp:20, max_hp: 20, defense:1, power:1}));

        //two parts of data aren't in a special struct - entity name and room it is in
        self.ecs_world.spawn(("Patron".to_string(), 0 as usize));
        self.ecs_world.spawn(("Thug".to_string(), 3 as usize, CombatStats{hp:10, max_hp:10, defense:1, power:1}));
        //item
        self.ecs_world.spawn(("Soda can".to_string(), 0 as usize, Item{}));
    }

    pub fn get_entities_in_room(&self, rid: usize) -> Vec<u64> {
        let mut list = Vec::new();
        for (id, (room_id)) in self.ecs_world.query::<(&usize)>()
        .without::<InBackpack>()
        .with::<String>()
        .iter() {
            if *room_id == rid {
                list.push(id.to_bits())
            }
        }
        return list;
    }

     //we store a list of ids and get the actual data with this separate function
    pub fn get_data_for_id(&self, id: u64) -> (u64, String, Option<Item>) {
        let ent = hecs::Entity::from_bits(id); //restore

        let name = self.ecs_world.get::<String>(ent).unwrap().to_string();
        let mut item: Option<Item> = None;

        if self.ecs_world.get::<Item>(ent).is_ok() {
            //need to dereference it
            item = Some(*self.ecs_world.get::<Item>(ent).unwrap())
        }
        
        return (id, name, item);
        
        //return format!("{} {}", id, name);
    }

    pub fn items_in_inventory(&self) -> Vec<(u64, String)>{
        let mut data = Vec::new();
        //test
        for (id, (item, backpack)) in &mut self.ecs_world.query::<(&Item, &InBackpack)>().iter(){
            //log!("{}", &format!("Item in inventory: {}", self.ecs_world.get::<&str>(id).unwrap().to_string()));
            //log!("{}", &format!("ID: {:?}", id));
            //ids.push(id.to_bits());
            let name = self.ecs_world.get::<String>(id).unwrap().to_string();
            data.push((id.to_bits(), name));
        }
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


    //a very simple test, akin to flipping a coin or throwing a d2
    fn make_test_d2(&self, skill: u32) -> Vec<bool> {
        let mut rolls = Vec::new();
        for _ in 0..10-skill { // exclusive of end
            rolls.push(rand::random()) // generates a boolean
        }
        return rolls
    }

    pub fn attack(&self, target: &Entity) {
        let res = self.make_test_d2(1);
        let sum = res.iter().filter(|&&b| b).count(); //iter returns references and filter works with references too - double indirection
        //game_message(&format!("Test: {} sum: {{g{}", Rolls(res), sum));

        if sum >= 5 {
            //game_message(&format!("Attack hits!"));

            //deal damage
            // the mut here is obligatory!!!
            let mut stats = self.ecs_world.get_mut::<CombatStats>(*target).unwrap();
            stats.hp = stats.hp - 2; // - offensive_bonus;
            log!("{}", &format!("Dealt 2 damage"));
            //game_message(&format!("Dealt {{r{}}} damage", 2+offensive_bonus));
            
            //can't remove dead here due to borrow checker
        } else {
            log!("Attack missed!");
        }
    }

}