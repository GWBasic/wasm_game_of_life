//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use array2d::Array2D;

wasm_bindgen_test_configure!(run_in_browser);

extern crate wasm_game_of_life;
use wasm_game_of_life::Universe;

#[cfg(test)]
pub fn input_spaceship() -> Universe {
    let mut universe = Universe::new(6,6);
    let rows = vec![
    	vec![0, 0, 0, 0, 0, 0],
    	vec![0, 0, 1, 0, 0, 0],
    	vec![0, 0, 0, 1, 0, 0],
    	vec![0, 1, 1, 1, 0, 0],
    	vec![0, 0, 0, 0, 0, 0],
    	vec![0, 0, 0, 0, 0, 0],
    ];
    universe.set_cells(Array2D::from_rows(&rows));
    universe
}

#[cfg(test)]
pub fn expected_spaceship() -> Universe {
    let mut universe = Universe::new(6,6);
    let rows = vec![
    	vec![0, 0, 0, 0, 0, 0],
    	vec![0, 0, 0, 0, 0, 0],
    	vec![0, 1, 0, 1, 0, 0],
    	vec![0, 0, 1, 1, 0, 0],
    	vec![0, 0, 1, 0, 0, 0],
    	vec![0, 0, 0, 0, 0, 0],
    ];
    universe.set_cells(Array2D::from_rows(&rows));
    universe
}

/*// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}*/

#[wasm_bindgen_test]
pub fn test_tick() {
    // Let's create a smaller Universe with a small spaceship to test!
    let mut input_universe = input_spaceship();

    // This is what our spaceship should look like
    // after one tick in our universe.
    let expected_universe = expected_spaceship();

    // Call `tick` and then see if the cells in the `Universe`s are the same.
    input_universe.tick();

    assert_eq!(&input_universe, &expected_universe);
}