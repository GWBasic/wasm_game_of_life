// TODO: There's probably a better way to do this
#[path = "universe.rs"] mod universe;
#[path = "web_sys_mixins.rs"] mod web_sys_mixins;

use wasm_bindgen::JsCast;

extern crate web_sys;
use web_sys::CanvasRenderingContext2d;
use web_sys::Element;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlElement;
use web_sys::HtmlInputElement;
use web_sys::MouseEvent;
use web_sys::Window;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;
use std::rc::Weak;

use universe::Universe;
use web_sys_mixins::HtmlExt;
use web_sys_mixins::IntervalSubscription;
use web_sys_mixins::RegisteredHtmlEvent;
use web_sys_mixins::WindowExt;

const CELL_SIZE: u32 = 10; // px
const WIDTH: u32 = 96;
const HEIGHT: u32 = 64;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

thread_local! {
    static CONSTANTS: Constants = Constants::new();
    
}

struct Constants {
	grid_color: JsValue,
	dead_color: JsValue,
	alive_color: JsValue,
}

pub struct Ui {
	welf: Weak<RefCell<Option<Ui>>>,
	window: Window,
	canvas_element: Element,
	canvas: HtmlCanvasElement,
	context: CanvasRenderingContext2d,
	play_pause_button: HtmlElement,
	ticks_per_second_input: HtmlInputElement,
	animation_id: Option<i32>,
	drawn_generation: i64,
	universe: Universe,
	render_loop_closure: Rc<RefCell<Option<Closure<dyn std::ops::FnMut(i32)>>>>,
	
	timer: Option<IntervalSubscription>,
	
	//timer: Option<i32>,
	//timer_closure: wasm_bindgen::closure::Closure<dyn std::ops::FnMut()>,

	#[allow(dead_code)]
	play_pause_button_event: RegisteredHtmlEvent<'static>,
	
	#[allow(dead_code)]
	clear_button_event: RegisteredHtmlEvent<'static>,
	
	#[allow(dead_code)]
	randomize_button_event: RegisteredHtmlEvent<'static>,
	
	#[allow(dead_code)]
	ticks_per_second_input_event: RegisteredHtmlEvent<'static>,
	
	#[allow(dead_code)]
	canvas_click_event: RegisteredHtmlEvent<'static>
}


impl Constants {
    pub fn new() -> Constants {
    	Constants {
			grid_color: JsValue::from_str("#CCCCCC"),
			dead_color: JsValue::from_str("#FFFFFF"),
			alive_color: JsValue::from_str("#000000"),
    	}
	}
}

impl Ui {
    pub fn new() -> Rc<RefCell<Option<Ui>>> {
    	let s: Rc<RefCell<Option<Ui>>> = Rc::new(RefCell::new(None));

	    // get window/document
	    let window = web_sys::window().expect("Could not get window");
	    let document = window.document().expect("Could not get document");
    
    	let canvas_element = document.get_element_by_id("game-of-life-canvas")
    		.expect("Could not get game-of-life-canvas element");
    	
    	let play_pause_button = document.get_element_by_id("play-pause")
    		.expect("Could not get the play-pause button")
    		.dyn_into::<web_sys::HtmlElement>()
    		.expect("Expected a button");
    	
    	let clear_button = document.get_element_by_id("clear")
    		.expect("Could not get the clear button")
    		.dyn_into::<web_sys::HtmlElement>()
    		.expect("Expected a button");
    	
    	let randomize_button = document.get_element_by_id("randomize")
    		.expect("Could not get the randomize button")
    		.dyn_into::<web_sys::HtmlElement>()
    		.expect("Expected a button");
    	
    	let ticks_per_second_input = document.get_element_by_id("tickspersecond")
    		.expect("Could not get the ticks per second slider")
    		.dyn_into::<web_sys::HtmlInputElement>()
    		.expect("Expected a slider");
    		
    	let canvas = canvas_element.clone().dyn_into::<web_sys::HtmlCanvasElement>()
    		.unwrap();
    		
    	canvas.set_height((CELL_SIZE + 1) * HEIGHT + 1);
		canvas.set_width((CELL_SIZE + 1) * WIDTH + 1);
    	
    	let context = canvas.get_context("2d")
    		.expect("Could not get 2d context")
    		.expect("Could not get 2d context")
        	.dyn_into::<web_sys::CanvasRenderingContext2d>()
        	.unwrap();
    
    	// TODO: Hack for requestAnimationFrame loop
		// https://github.com/bzar/wasm-pong-rs/blob/master/src/lib.rs
		let render_loop_s = Rc::downgrade(&s);
	    let render_loop_closure = Rc::new(RefCell::new(None));
    	let g = render_loop_closure.clone();
    	*g.borrow_mut() = Some(Closure::wrap(Box::new(move |_| {
			(*(render_loop_s.upgrade().unwrap().borrow_mut())).as_mut().unwrap().render_loop();
    	}) as Box<dyn FnMut(i32)>));    	

/*
    	let timer_closure_s = Rc::downgrade(&s);
    	let timer_closure = Closure::wrap(Box::new(move || {
			(*(timer_closure_s.upgrade().unwrap().borrow_mut())).as_mut().unwrap().universe.tick();
		}) as Box<dyn FnMut()>);*/
	
    	let play_pause_button_s = Rc::downgrade(&s);
    	let clear_button_s = Rc::downgrade(&s);
    	let randomize_button_s = Rc::downgrade(&s);
    	let ticks_per_second_s = Rc::downgrade(&s);
    	let canvas_s = Rc::downgrade(&s);
    	    
    	let mut f = Ui {
    		welf: Rc::downgrade(&s),
    		window,
    		context,
			animation_id: None,
			drawn_generation: -1,
			universe: Universe::new(WIDTH, HEIGHT),
			render_loop_closure,
			timer: None,
			//timer_closure,
			
			play_pause_button_event : play_pause_button.events().add_event_listener("click", Box::new(move |_| {
				(*(play_pause_button_s.upgrade().unwrap().borrow_mut())).as_mut().unwrap().play_pause();
			})).unwrap(),
			
    		play_pause_button,
    		
    		clear_button_event: clear_button.events().add_event_listener("click", Box::new(move |_| {
    			(*(clear_button_s.upgrade().unwrap().borrow_mut())).as_mut().unwrap().clear();
    		})).unwrap(),
    		
			randomize_button_event: randomize_button.events().add_event_listener("click", Box::new(move |_| {
				(*(randomize_button_s.upgrade().unwrap().borrow_mut())).as_mut().unwrap().randomize();
			})).unwrap(),
			
			ticks_per_second_input_event: ticks_per_second_input.events().add_event_listener("click", Box::new(move |_| {
				(*(ticks_per_second_s.upgrade().unwrap().borrow_mut())).as_mut().unwrap().update_ticks_per_second();
			})).unwrap(),

    		ticks_per_second_input,
    		
    		canvas_click_event: canvas.events().add_event_listener("click", Box::new(move |event| {
				let mouse_event = event.dyn_into::<MouseEvent>().unwrap();
				(*(canvas_s.upgrade().unwrap().borrow_mut())).as_mut().unwrap().canvas_click(mouse_event);
			})).unwrap(),
    		canvas_element,
    		canvas,
    	};
    	
    	f.universe.randomize();
    	f.pause();
    	
    	s.replace(Some(f));
    	s
	}
	
	pub fn draw_grid(&self) {
		self.context.begin_path();
		CONSTANTS.with(|c| self.context.set_stroke_style(&c.grid_color));
		
		// Vertical lines.
		for i in 0..self.universe.width() {
			self.context.move_to((i * (CELL_SIZE + 1) + 1) as f64, 0.0);
			self.context.line_to((i * (CELL_SIZE + 1) + 1) as f64, ((CELL_SIZE + 1) * self.universe.height() + 1) as f64);
		}

		// Horizontal lines.
		for j in 0..self.universe.height() {
			self.context.move_to(0.0,                           (j * (CELL_SIZE + 1) + 1) as f64);
			self.context.line_to(((CELL_SIZE + 1) * self.universe.width() + 1) as f64, (j * (CELL_SIZE + 1) + 1) as f64);
		}

		self.context.stroke();
	}
	
	fn draw_cells(&self) {		
		CONSTANTS.with(|c| {
			for row in 0..self.universe.height() {
   		    	for col in 0..self.universe.width() {
           		    let cell = self.universe.cell_at(row, col);
                
               		let fill_style = match cell {
          				true => &c.alive_color,
               			false => &c.dead_color
            		};
                
       		        self.context.set_fill_style(&fill_style);
           		    self.context.fill_rect(
		    		    (col * (CELL_SIZE + 1) + 1) as f64,
		        		(row * (CELL_SIZE + 1) + 1) as f64,
			        	CELL_SIZE as f64,
				        CELL_SIZE as f64);
				}
       	    }
        });
	}
	
	fn render_loop(& mut self) {
		let generation = self.universe.generation();

		if generation != self.drawn_generation {
			self.draw_cells();
			self.drawn_generation = generation;
		}
		
		if self.timer.is_some() {
	    	let animation_id = self.window.request_animation_frame(
    			self.render_loop_closure.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        		.expect("should register `requestAnimationFrame` OK");
			self.animation_id = Some(animation_id);
		} else {
			self.animation_id = None;
		}
	}
	
	fn reset_timer(& mut self) {
		/*match self.timer {
			Some(t) => self.window.clear_interval_with_handle(t),
			None => {}
		}
		
		let delay = 1.0 / self.ticks_per_second_input.value_as_number();
		let timer = self.window.set_interval_with_callback_and_timeout_and_arguments_0(
			self.timer_closure.as_ref().unchecked_ref(),
			(delay * 1000.0) as i32)
			.expect("Timer not set");
			
		self.timer = Some(timer);*/
		
    	let welf = self.welf.clone();
		let delay = (1.0 / self.ticks_per_second_input.value_as_number() * 1000.0) as i32;
		let timer = self.window.set_interval(Box::new(move || {
				(*(welf.upgrade().unwrap().borrow_mut())).as_mut().unwrap().universe.tick();
			}),
			delay)
		.expect("Timer not set");
		
		self.timer = Some(timer);
	}
	
	fn play_pause(& mut self) {
		match self.timer {
			Some(_) => self.pause(),
			None => self.play()
		}
	}
	
	fn play(& mut self) {
		self.play_pause_button.set_text_content(Some("⏸"));
		self.reset_timer();
		self.render_loop();
	}

	fn pause(& mut self) {
		self.play_pause_button.set_text_content(Some("▶"));

		self.draw_grid();
		self.draw_cells();
		
		self.timer = None;
		
		match self.animation_id {
			Some(a) => {
				self.window.cancel_animation_frame(a)
					.expect("Can't cancel animcation");
				self.animation_id = None;
			},
			None => {}
		}
	}
	
	fn clear(& mut self) {
    	self.universe = Universe::new(WIDTH, HEIGHT);
		self.draw_cells();
	}
	
	fn randomize(& mut self) {
		self.universe.randomize();
		self.draw_cells();
	}
	
	fn update_ticks_per_second(& mut self) {
		if self.timer.is_some() {
			self.reset_timer();
		}
	}
	
	fn canvas_click(& mut self, mouse_event: MouseEvent) {
		let bounding_rect = self.canvas_element.get_bounding_client_rect();
		
		let scale_x = (self.canvas.width() as f64) / bounding_rect.width();
		let scale_y = (self.canvas.height() as f64) / bounding_rect.height();

		let canvas_left = (mouse_event.client_x() as f64 - bounding_rect.left()) * scale_x;
		let canvas_top = (mouse_event.client_y() as f64 - bounding_rect.top()) * scale_y;

		let row = min(canvas_top as u32 / (CELL_SIZE + 1), HEIGHT - 1);
		let col = min(canvas_left as u32 / (CELL_SIZE + 1), WIDTH - 1);

		self.universe.toggle_cell(row, col);

		self.draw_cells();
	}
}