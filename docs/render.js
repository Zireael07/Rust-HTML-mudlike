//JS starts here
import * as rust from './rust_web_text_based.js';

var universe, g_wasm; // Can't be initialized yet because WASM is not ready

//Toki Pona words
var wordlist = {
    "e": "object marker", //like '(w)o' in Japanese
    "ijo": "thing",
    "ilo": "tool, device",
    "jan": "person",
    "kili": "fruit",
    "li": "pred marker", //sort of like 'wa' in Japanese
    "lili": "small",
    "lukin": "eye, to look",
    "mama": "parent",
    "meli": "woman",
    "mi": "mine",
    "mije": "man",
    "moku": "eat, food",
    "ona": "3rd person sg/pl",
    "pali": "to do/work/make",
    "pana": "give, send",
    "pipi": "bug(s)",
    "pona": "good",
    "soweli": "animal",
    "suli": "great",
    "suwi": "sweet",
    "telo": "water, fluid",
    "utala": "fight",
    "wawa": "strong",
    "waso": "bird",
}




//this calls back to Rust
function process(cmd) {
    universe.process(cmd);
    //now redraw!
    draw();
}


function exitClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    //console.log("ID: ", i);
    //console.log("Clicked exit");
    process("go " + i);
}

function itemClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    process("get " + i);
}

function itemBackpackClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    
    var output = document.getElementById("game").innerHTML;
    output += "\n\n";
    //TODO: only display use button if item is usable
    output += `<button class="inv_use_button" id=ent-${id}>Use</button>` + " " + `<button class="inv_drop_button" id=ent-${id}>Drop</button>`

    document.getElementById("game").innerHTML = output;

    let inv = universe.display_inventory();
    addHandlers(inv);
}


function dropClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    process("drop " + i);
}

function useClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    process("use " + i);
}

function inventoryOpenClick(button, inv) {
    var str = 'You are carrying: ';
    var output = document.getElementById("game").innerHTML;
    var is_open = output.indexOf(str);

    //don't append if already open
    if (is_open == -1) {
        //append to game view
        output += '\n\n ';
        output += str;

        //nice formatting
        // if (inv.length == 1) {
        //     var item = inv[0];
        // }
        // else {
            for (var i=0; i<inv.length;i++) {
                var item = inv[i];
                output += `<button class="it_inv_button" id=ent-${item[0]}>${item[1]}</button>` + " ";
            }
        //}


        document.getElementById("game").innerHTML = output;

        addHandlers(inv);
    }

   
}

function npcClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    
    process("npc_interact " + i);
    
    //test generating sentences
    //https://observablehq.com/@dhowe/tut-rita-ngrams
    var rm = RiTa.markov(3);

    //sentences from https://rnd.neocities.org/tokipona/
    //catch: sina li suli would be sina suli in standard Toki Pona
    rm.addText("ona li suli.\
    kili li pona.\
    sina li suli.\
    soweli lili suwi.\
    mama mi li pona.\
    jan utala li wawa.\
    jan lili mi li suwi.\
    soweli lili li wawa ala.\
    meli mi li pona.\
    mije sina li suli.\
    soweli ale li pona.\
    kili li moku suli.\
    jan lili li (pana e telo lukin).\
    ona li lukin e lipu.\
    soweli ike li utala e meli.\
    jan utala li moku e kili suli.\
    soweli lili li moku e telo.\
    mi telo e ijo suli.\
    jan wawa li pali e tomo.\
    jan pali li telo e kasi.\
    jan wawa li jo e kiwen suli.\
    waso lili li moku e pipi.\
    meli li toki e soweli, e waso.\
    jan pali li pona e ilo, li lukin e lipu.\
    jan pali li pana e moku pona."
    );
    var sentences = rm.generate(2);

    //append to game view
    var output = document.getElementById("game").innerHTML;
    output += '\n\n ';
    for (var i=0; i < sentences.length;i++) {
        var s = sentences[i];
        output += s + '\n';
        //gloss
        var tokens = RiTa.tokenize(s);
        for (var i=0; i<tokens.length;i++){
            var t = tokens[i];
            if (t != ".") {
                output += wordlist[t] + " ";
            }
            
        }
    }

    document.getElementById("game").innerHTML = output;

    let inv = universe.display_inventory();
    addHandlers(inv);
}


function draw() {
    var map = universe.get_current_map();
    var entities = universe.display_entities_in_room();
    let inv = universe.display_inventory();
    let messages = universe.display_messages();

    var output = map.desc + '\n\n';

    output = output + "You see here: ";
    for (var i=0; i < entities.length; i++){
        var entity = entities[i];
        if (entity[2]) {
            output = output + `<button class="it_button" id=ent-${entity[0]}>${entity[1]}</button>`;
        } else {
            output = output + `<button class="ent_button" id=ent-${entity[0]}>${entity[1]}</button>`;
        }
    }

    output = output + "\n\n";

    for (var i =0; i < map.exits.length; i++) {
        var exit = map.exits[i];

        // display names of known rooms
        var exit_room_id = exit[1];
        var exit_room = universe.get_room_id(exit_room_id);
        var exit_display = exit[0];
        if (exit_room.known) {
            exit_display = "To " + exit_room.name;
        }
        // those are backticks, not straight quotes!
        output = output + `<button class="exit_button" id=item-${exit_room_id}>${exit_display}</button> `;
    }

    output = output + '\n\n';
    
    for (var i=0; i<messages.length; i++) {
        var msg = messages[i];
        output = output + msg +'\n';
    }

    if (inv.length > 0) {
        output = output +  `<button class="inv_button" id=inventory>Inventory</button>`;
    }

    document.getElementById("game").innerHTML = output;

    addHandlers(inv);
}

function addHandlers(inv){
        // interactivity!
        var buttons = document.querySelectorAll(".exit_button");
        for (var i = 0; i < buttons.length; i++) {
            var button = buttons[i];
            button.onclick = function(e) { exitClick(e.target); }
        }
        var npc_buttons = document.querySelectorAll(".ent_button");
        for (var i = 0; i < npc_buttons.length; i++) {
            var button = npc_buttons[i];
            button.onclick = function(e) { npcClick(e.target); }
        }
    
        var it_buttons = document.querySelectorAll(".it_button");
        for (var i = 0; i < it_buttons.length; i++) {
            var button = it_buttons[i];
            button.onclick = function(e) { itemClick(e.target); }
        }

        var it_buttons = document.querySelectorAll(".it_inv_button");
        for (var i = 0; i < it_buttons.length; i++) {
            var button = it_buttons[i];
            button.onclick = function(e) { itemBackpackClick(e.target); }
        }
    
        if (inv.length > 0) {
            var inv_button = document.querySelector(".inv_button");
            inv_button.onclick = function(e) { inventoryOpenClick(e.target, inv); }
        }

        var drop_button = document.querySelector(".inv_drop_button");
        if (drop_button) {
            drop_button.onclick = function(e) { dropClick(e.target); }
        }
            

        var use_button = document.querySelector(".inv_use_button");
        if (use_button) {
            use_button.onclick = function(e) { useClick(e.target); }
        }
        
}

//logic shuffled to Rust (see load_datafiles())
//needs to be async to be able to use await
async function initGame(wasm) {
    universe = rust.Universe.new();
    //async/await again to load text data
    //workaround
    universe = await rust.load_datafile(universe);

    initRenderer(wasm);
}

function initRenderer(wasm) {
    //universe = rust.Universe.new();

    draw();

}

export { initGame }