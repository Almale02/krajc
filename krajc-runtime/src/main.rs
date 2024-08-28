use std::process::Command;

#[deny(unused_imports)]
#[deny(warnings)]
use krajc::prelude::*;
use krajc::{typed_addr::dupe, Lateinit};
use libloading::Library;
use stabby::libloading::StabbyLibrary;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let game = args.get(1).expect("didnt provide a game to run").clone();

    let game_path = get_game_plugin_path(game);

    // this is a really stupid bug in libloading, for some reason if i load a library with libloading, then when the function which called libloading finishes,
    // it just freezez, thats why it needs to run in a seperate thread because it is not present there
    // also i have implemented Send for Lateinit, not because it is safe, but because it is easier to use
    let x = unsafe { Library::new(game_path).unwrap() };

    let plugin = unsafe {
        StabbyLibrary::get_stabbied::<extern "C" fn() -> SystemPlugin>(&x, b"get_plugin").unwrap()
    }();

    //lib.set(plugin());

    let tokio = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    dbg!("here");

    tokio.block_on(run(plugin));

    // Use `taskkill` to terminate the current process
    let output = Command::new("taskkill")
        .args(["/F", "/PID", &std::process::id().to_string()])
        .status()
        .unwrap();
    dbg!("ran end");
}
