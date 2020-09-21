mod ui;

extern crate web_sys;
use wasm_bindgen::prelude::*;

use ui::Ui;

/*
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}*/

#[wasm_bindgen]
pub fn start() {
	let ui = Ui::new();
	std::mem::forget(ui);
}

    // New TODOs:
    // *1: State is forgotten (state.forget) so that it lives forever
    // *2: State is constructed first, then buttons loaded and events hooked up
    // *3: Not sure what the best way is to share the state value with the event handlers
    // *3a: event() function is part of a trait, single struct with add_listener()
    // 3b: StackOverflow post about why cloning the state won't work in add_listener()
    // 3a: "use" mixin that is option.use(|x| {}), for Rc<RefCell<Option and Weak
    // Or maybe a new type that encapsulates a weak reference so it's easy to use in closures
    // *4: Move the UI to a separate file (ui.rs)
    // *5: Rename "State" to "Ui"
    // *6: See if I can get rid of the lifetime parameter on State/Ui
    // *7: Clean up timer closure
    // *8: Clean up canvas closures
    // 9: Maybe publish crate to simplify HTML events?
    // 10: Static constants
    // *Declare state without events, use Rc<State>, register events on a clone, get rid of global state
    //* Ui is kept alive due to a cyclic reference, keep alive in a safer manner
