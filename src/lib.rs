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
	/*let ui =*/ Ui::new();
	
	// Ui currently has cyclic loops, leaks implicitly
	//ui.forget();
}

