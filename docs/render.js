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

function draw() {
    var map = universe.get_current_map();

    var output = map.desc + '\n\n';
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
}


function initRenderer(wasm) {
    universe = rust.Universe.new();

    draw();

}

export { initRenderer }