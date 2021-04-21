//JS starts here
import * as rust from './rust_web_text_based.js';

var universe, g_wasm; // Can't be initialized yet because WASM is not ready

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

function npcClick(button) {
    //extract id from item id
    var id = button.id;
    var reg = id.match(/(\d+)/); 
    var i = reg[0];
    
    
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
    jan lili li pana e telo lukin.\
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
    }
    document.getElementById("game").innerHTML = output;
}


function draw() {
    var map = universe.get_current_map();
    var entities = universe.display_entities_in_room();

    var output = map.desc + '\n\n';

    output = output + "You see here: ";
    for (var i=0; i < entities.length; i++){
        var entity = entities[i];
        output = output + `<button class="ent_button" id=ent-${entity[0]}>${entity[1]}</button>`;
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

    document.getElementById("game").innerHTML = output;

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