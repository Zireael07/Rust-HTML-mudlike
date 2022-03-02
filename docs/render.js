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

function examineClick() {
    //append to game view
    var output = document.getElementById("game").innerHTML;
    output += '\n\n ';

    var examine = universe.examine();
    output += examine;

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

    output = output + `<button class="exa_button" id=examine>Examine</button>`;

    document.getElementById("game").innerHTML = output;

    addHandlers(inv);
}

//--------------------------------
// SVG map toggle
function showMap() {
    console.log("Toggle map");
    if (!document.querySelector(".map").classList.contains('visible')) {
        document.querySelector(".map").classList.toggle('visible', true)
        //update the current node's name
        var n = document.getElementById("node1");
        //console.log(n.childNodes);
        n.childNodes[0].textContent = universe.get_current_map().name;
        n.childNodes[4].textContent = universe.get_current_map().name;
        //n.setAttributeNS(null, "text", universe.get_current_map().name);

        // display known exit rooms
        var others = universe.get_known_exits();
        var more = universe.get_more_known();
        //console.log(others);
        for (var i=0; i<others.length; i++){
            addNode(i+2, others[i].name);
        }

        console.log(more);
        for (var i=0; i<more.length; i++){
            addNodeMore(others.length+2+i, more[i]);
        }
        

        updateMapSize(others.length, more.length);
    } else {
        document.querySelector(".map").classList.toggle('visible', false);

        //remove all children of "graph" that are not "node1"
        //getElementsByTagName returns a HTML collection
        var collection = document.querySelector(".graph").getElementsByTagName("g");
        var graph_els = Array.from(collection);
        console.log(graph_els);
        graph_els.shift(); //we remove the first entry in the list, which should be node1

        for (var i=0; i<graph_els.length; i++) {
            if (graph_els[i].id.indexOf("node") != -1) {
                graph_els[i].remove() //nuke the element altogether
            }
        }
    }
    
}

function setupSVGNode(i,name) {
    var svg = document.querySelector("#svg");
    var svgNS = svg.namespaceURI;
    var g = document.createElementNS(svgNS, 'g');
    g.setAttributeNS(null, "id", "node"+i);
    g.setAttributeNS(null, "class", "node");
    var t = document.createElementNS(svgNS, 'title');
    t.textContent = name;
    var ell = document.createElementNS(svgNS,'ellipse');
    ell.setAttributeNS(null, "rx", 35);
    ell.setAttributeNS(null, "ry", 18);
    ell.setAttributeNS(null, "style", "fill:none;stroke:black;");
    var tex = document.createElementNS(svgNS, 'text');
    tex.setAttributeNS(null, "text-anchor", 'middle');
    tex.setAttributeNS(null, 'style', "font-family:Times New Roman;font-size:14.00;");
    tex.textContent = name;
    return [ell, tex, g, t];
}

// i is 2 for the first one, because "node1" is the current room
function addNode(i, name) {
    var elements = setupSVGNode(i, name);
    var ell = elements[0];
    //positioning
    //36 for first, 36+35*2 = 36+70 for second, etc
    //subtract 2 because we start from 2
    var x = 36+72*(i-2);
    ell.setAttributeNS(null, "cx", x);
    ell.setAttributeNS(null, "cy", -20);
    var tex = elements[1];
    //positioning
    tex.setAttributeNS(null, "x", x);
    tex.setAttributeNS(null, "y", -15); //+5 compared to ellipse

    //add to tree
    var graph = document.querySelector("#graph0");
    var g = elements[2];
    graph.appendChild(g);
    g.appendChild(elements[3]);
    g.appendChild(ell);
    g.appendChild(tex);
}

// for adding the second row of rooms
function addNodeMore(i, entry) {
    // entry is [index of exit room (i.e. the one above), room]
    var name = entry[1].name;
    var elements = setupSVGNode(i, name);
    var ell = elements[0];
    //positioning
    //position below the room that led to it
    //36 for first, 36+35*2 = 36+70 for second, etc
    var x = 36+72*(entry[0]);
    ell.setAttributeNS(null, "cx", x);
    ell.setAttributeNS(null, "cy", 20);
    var tex = elements[1];
    //positioning
    tex.setAttributeNS(null, "x", x);
    tex.setAttributeNS(null, "y", 25); //+5 compared to ellipse

    //add to tree
    var graph = document.querySelector("#graph0");
    var g = elements[2];
    graph.appendChild(g);
    g.appendChild(elements[3]);
    g.appendChild(ell);
    g.appendChild(tex);
}

function updateMapSize(length, more) {
    var svg = document.querySelector("#svg");
    var svgNS = svg.namespaceURI;
    var size = 100;
    var height = 100;
    
    if (length > 0) {
        // 80 is the size of the ellipse, roughly
        size = 80*length;
    }
    if (more >0) {
        height = 200;
    }

    if (height > 100) {
        svg.setAttributeNS(null, "height", 200);
    }

    svg.setAttributeNS(null, "width", size);
    //svg.setAttributeNS(null, "viewBox", "0.00 0.00 100.00 "+ size +".00");
    var bg = svg.querySelector("polygon");
    bg.setAttributeNS(null, "points", "-4,4 -4,-"+height+" "+ size+",-"+height+" "+size+",4 -4,4");
    if (height > 100) {
        var graph = svg.querySelector("#graph0")
        graph.setAttributeNS(null, "transform", "scale(1 1) rotate(0) translate(4 200)");
        //transform all the nodes up
        var nodes = svg.querySelectorAll(".node");
        //console.log(nodes);
        for (var i=0; i < nodes.length; i++) {
            nodes[i].setAttributeNS(null, "transform", "translate(0,-50)");
        }

    }

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

        var exa_button = document.querySelector(".exa_button");
        if (exa_button) {
            exa_button.onclick = function(e) { examineClick(); }
        }
        
        //map handler
        var map_button = document.querySelector(".nav");
        if (map_button) {
            map_button.onclick = function(e) { showMap(); }
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