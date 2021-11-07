//JS starts here
import * as rust from './rust_web_text_based.js';

var universe, g_wasm; // Can't be initialized yet because WASM is not ready

//Toki Pona words
var wordlist = {
    "akesi": "reptile",
    "ala": "not",
    "alasa": "hunt",
    "ale": "all, every",
    "ante": "change(d), different",
    "anu": "or",
    "awen": "keep, stay",
    "e": "object marker", //like '(w)o' in Japanese
    "en": "and",
    "ike": "bad",
    "ijo": "thing",
    "ilo": "tool, device",
    "jaki": "yucky",
    "jan": "person",
    "jo": "have",
    "kala": "fish",
    "kalama": "sound, noise",
    "kama": "come, arrive",
    "kasi": "plant(s)",
    "kili": "fruit",
    "kin": "also",
    "kiwen": "stone, rock, hard",
    "ken": "can, able",
    "kulupu": "group",
    "la": "context marker",
    "lape": "sleep, rest",
    "lawa": "lead(er)",
    "li": "pred marker", //sort of like 'wa' in Japanese
    "lili": "small",
    "lipu": "book, paper",
    "linja": "line, hair",
    "lon": "exist",
    "luka": "hand, five",
    "lukin": "eye, to look",
    "ma": "earth, land",
    "mama": "parent",
    "mani": "money",
    "meli": "woman",
    "mi": "I, mine",
    "mije": "man",
    "moku": "eat, food",
    "moli": "death, dead(ly)",
    "musi": "fun, game",
    "mute": "a lot",
    "nasa": "weird, strange",
    "ni": "this, that",
    "nimi": "name, word",
    "o": "[exclam]",
    "ona": "3rd person sg/pl",
    "pakala": "damage, destroy",
    "pali": "to do/work/make",
    "pana": "give, send",
    "pi": "of",
    "pilim": "feeling, heart",
    "pipi": "bug(s)",
    "poka": "next to, side",
    "pona": "good",
    "silim": "feel", //toki pona has no 'f'
    "sina": "you",
    "seli": "heat",
    "seme": "what",
    "sona": "know",
    "soweli": "animal",
    "suli": "great",
    "suwi": "sweet",
    "tan": "from, by",
    "taso": "but",
    "tawa": "go",
    "telo": "water, fluid",
    "tenpo": "time",
    "toki": "talk",
    "tomo": "house, room",
    "tu": "two",
    "utala": "fight",
    "wawa": "strong",
    "waso": "bird",
    "wile": "want, need",
    "weka": "remove",
    //punctuation so that it doesn't show up as undefined
    ".": "",
    ",": "",
    "(": "",
    ")": "",
    "?": "",
    "!": "",
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

        output += 'You are wearing: ';
        var equip = inv[0];
        for (var i=0; i<equip.length;i++){
            var item = equip[i];
            output += `<button class="it_inv_button" id=ent-${item[0]}>${item[1]}</button>` + " ";
        }

        output += '\n' + str;

        //nice formatting
        // if (inv.length == 1) {
        //     var item = inv[0];
        // }
        // else {
        var carried = inv[1];
        for (var i=0; i<carried.length;i++) {
            var item = carried[i];
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
    var ques = false;
    //seller can ask you questions
    if (i == "4"){
        ques = true;
    }

    // this is a basic Rust implementation that has some simple rules in addition to Markov chain
    var sentence = universe.get_sentences(ques);
    //hack for now
    var sentences = [sentence]

    //append to game view
    var output = document.getElementById("game").innerHTML;
    output += '\n\n ';
    for (var i=0; i < sentences.length;i++) {
        var s = sentences[i];
        output += s + '\n';
        //gloss
        var tokens = universe.get_tokens(s);
        for (var i=0; i<tokens.length;i++){
            var t = tokens[i];
            if (t != "") {
                if (i < tokens.length-2) {
                    output += wordlist[t] + " - ";
                } else {
                    output += " " + wordlist[t];
                }
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
            output = output + `<button class="it_button" id=ent-${entity[0]}>${entity[1]} [${entity[3]}]</button>`;
        } else {
            output = output + `<button class="ent_button" id=ent-${entity[0]}>${entity[1]} [${entity[3]}]</button>`;
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
            exit_display = "To " + exit_room.name + "["+exit[0]+"]";
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