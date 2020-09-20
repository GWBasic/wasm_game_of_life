mod utils;
mod web_sys_mixins;

extern crate fixedbitset;
use fixedbitset::FixedBitSet;

pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
    generation: i64
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [-1i32, 0, 1].iter().cloned() {
            for delta_col in [-1i32, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let mut neighbor_row = row as i32 + delta_row;
                if neighbor_row < 0 {
                	neighbor_row += self.height as i32;
                }
                
                let mut neighbor_col = column as i32 + delta_col;
                if neighbor_col < 0 {
                	neighbor_col += self.width as i32;
                }
                
                let idx = self.get_index(neighbor_row as u32, neighbor_col as u32);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

/// Public methods, exported to JavaScript.
impl Universe {
    pub fn tick(&mut self) {

        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
            
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

				let next_cell = match (cell, live_neighbors) {

                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
				    (true, x) if x < 2 => false,

                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
				    (true, 2) | (true, 3) => true,

                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
				    (true, x) if x > 3 => false,

                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
				    (false, 3) => true,

                    // All other cells remain in the same state.
				    (otherwise, _) => otherwise
				};

            	next.set(idx, next_cell);
            }
        }

        self.cells = next;
        self.generation += 1;
    }

    pub fn new(width: u32, height: u32) -> Universe {

        let size = (width * height) as usize;
	    let cells = FixedBitSet::with_capacity(size);

        Universe {
            width,
            height,
            cells,
            generation: 0
        }
    }
    
    pub fn randomize(&mut self) {
        let size = (self.width * self.height) as usize;

    	for i in 0..size {
        	self.cells.set(i, js_sys::Math::random() < 0.5);
	    }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cell_at(&self, row: u32, column: u32) -> bool {
        self.cells[self.get_index(row, column)]
    }
    
    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.set(idx, !self.cells[idx]);
    }
    
    pub fn generation(&self) -> i64 {
    	self.generation
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let symbol = if cell { '◼' } else { '◻' };
                write!(f, "{}", symbol)?;
			}
            write!(f, "\n")?;
		}
		
        Ok(())
    }
}

impl PartialEq for Universe {
    fn eq(&self, other: &Self) -> bool {
        self.height == other.height &&
        self.width == other.width &&
        self.cells == other.cells
    }
}
impl Eq for Universe {}

impl fmt::Debug for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
         .field("width", &self.width)
         .field("height", &self.height)
         .field("cells", &self.to_string())
         .finish()
    }
}