use super::log;
use super::{Universe, Room, Exit, DataMaster, 
    Item, InBackpack
};

use hecs::Entity;

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

        //two parts of data aren't in a special struct - entity name and room it is in
        self.ecs_world.spawn(("Patron".to_string(), 0 as usize));
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

    pub fn items_in_inventory(&self) -> Vec<String>{
        let mut names = Vec::new();
        //test
        for (id, (item, backpack)) in &mut self.ecs_world.query::<(&Item, &InBackpack)>().iter(){
            //log!("{}", &format!("Item in inventory: {}", self.ecs_world.get::<&str>(id).unwrap().to_string()));
            //log!("{}", &format!("ID: {:?}", id));
            //ids.push(id.to_bits());
            let name = self.ecs_world.get::<String>(id).unwrap().to_string();
            names.push(name);
        }
        return names;
    }

    pub fn pickup_item(&mut self, item: &Entity) {
        self.ecs_world.insert_one(*item, InBackpack{});
        //self.items_in_inventory();
    }

}