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

use std::cell::RefCell;
use std::cmp::min;
use std::rc::Rc;
use std::rc::Weak;

use universe::Universe;
use web_sys_mixins::AnimationFrameRequester;
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
	welf: Option<Weak<RefCell<Ui>>>,
	window: Window,
	canvas_element: Element,
	canvas: HtmlCanvasElement,
	context: CanvasRenderingContext2d,
	play_pause_button: HtmlElement,
	clear_button: HtmlElement,
	randomize_button: HtmlElement,
	ticks_per_second_input: HtmlInputElement,
	animation_id: Option<i32>,
	drawn_generation: i64,
	universe: Universe,
	
	animation_frame_requester: Option<AnimationFrameRequester>,
	
	timer: Option<IntervalSubscription>,
	
	#[allow(dead_code)]
	play_pause_button_event: Option<RegisteredHtmlEvent<'static>>,
	
	#[allow(dead_code)]
	clear_button_event: Option<RegisteredHtmlEvent<'static>>,
	
	#[allow(dead_code)]
	randomize_button_event: Option<RegisteredHtmlEvent<'static>>,
	
	#[allow(dead_code)]
	ticks_per_second_input_event: Option<RegisteredHtmlEvent<'static>>,
	
	#[allow(dead_code)]
	canvas_click_event: Option<RegisteredHtmlEvent<'static>>
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
    pub fn new() -> Rc<RefCell<Ui>> {
	    // get window/document
	    let window = web_sys::window().expect("Could not get window");
	    let document = window.document().expect("Could not get document");
    
    	let canvas_element = document.get_element_by_id("game-of-life-canvas")
    		.expect("Could not get game-of-life-canvas element");
    		
    	let canvas = canvas_element.clone().dyn_into::<web_sys::HtmlCanvasElement>()
    		.unwrap();
    		
    	canvas.set_height((CELL_SIZE + 1) * HEIGHT + 1);
		canvas.set_width((CELL_SIZE + 1) * WIDTH + 1);
    	
    	let context = canvas.get_context("2d")
    		.expect("Could not get 2d context")
    		.expect("Could not get 2d context")
        	.dyn_into::<web_sys::CanvasRenderingContext2d>()
        	.unwrap();
    	    
    	let self_rc = Rc::new(RefCell::new(Ui {
    		context,
			animation_id: None,
			drawn_generation: -1,
			universe: Universe::new(WIDTH, HEIGHT),
			timer: None,
			
    		play_pause_button: document.get_element_by_id("play-pause")
	    		.expect("Could not get the play-pause button")
    			.dyn_into::<web_sys::HtmlElement>()
    			.expect("Expected a button"),
    			
    		clear_button: document.get_element_by_id("clear")
    			.expect("Could not get the clear button")
    			.dyn_into::<web_sys::HtmlElement>()
	    		.expect("Expected a button"),
    			
    		randomize_button: document.get_element_by_id("randomize")
    			.expect("Could not get the randomize button")
    			.dyn_into::<web_sys::HtmlElement>()
	    		.expect("Expected a button"),

    		ticks_per_second_input: document.get_element_by_id("tickspersecond")
    			.expect("Could not get the ticks per second slider")
    			.dyn_into::<web_sys::HtmlInputElement>()
	    		.expect("Expected a slider"),
    		
    		canvas_element,
    		canvas,
    		window,

    		welf: None,
			animation_frame_requester: None,
			play_pause_button_event: None,
			clear_button_event: None,
			randomize_button_event: None,
			ticks_per_second_input_event: None,
    		canvas_click_event: None,
    	}));

		{
			let mut s = self_rc.borrow_mut();

			let welf = Rc::downgrade(&self_rc);
			(*s).welf = Some(welf.clone());
		
			let animation_s = Rc::downgrade(&self_rc);
			(*s).animation_frame_requester = Some((*s).window.prepare_animation_frame_callback(Box::new(move |_| {
				(*(animation_s.upgrade().unwrap().borrow_mut())).render_loop();
			})));

    		let play_pause_button_s = Rc::downgrade(&self_rc);
			(*s).play_pause_button_event = Some((*s).play_pause_button.events().add_event_listener_state("click", &welf, Box::new(move |w, _| {
				(*(w.upgrade().unwrap().borrow_mut())).play_pause();
			})).unwrap());

    		let clear_button_s = Rc::downgrade(&self_rc);
   			(*s).clear_button_event = Some((*s).clear_button.events().add_event_listener("click", Box::new(move |_| {
				(*(clear_button_s.upgrade().unwrap().borrow_mut())).clear();
	   		})).unwrap());
    		    		
	    	let randomize_button_s = Rc::downgrade(&self_rc);
			(*s).randomize_button_event = Some((*s).randomize_button.events().add_event_listener("click", Box::new(move |_| {
				(*(randomize_button_s.upgrade().unwrap().borrow_mut())).randomize();
			})).unwrap());
			
    		let ticks_per_second_s = Rc::downgrade(&self_rc);
			(*s).ticks_per_second_input_event = Some((*s).ticks_per_second_input.events().add_event_listener("click", Box::new(move |_| {
				(*(ticks_per_second_s.upgrade().unwrap().borrow_mut())).update_ticks_per_second();
			})).unwrap());

    		let canvas_s = Rc::downgrade(&self_rc);
    		(*s).canvas_click_event = Some((*s).canvas.events().add_event_listener("click", Box::new(move |event| {
				let mouse_event = event.dyn_into::<MouseEvent>().unwrap();
				(*(canvas_s.upgrade().unwrap().borrow_mut())).canvas_click(mouse_event);
			})).unwrap());
    	
    		(*s).universe.randomize();
    		(*s).pause();
    	}
    	
    	self_rc
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
			self.context.move_to(0.0, (j * (CELL_SIZE + 1) + 1) as f64);
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
			match &self.animation_frame_requester {
				Some(animation_frame_requester) => {
					let animation_id = animation_frame_requester.request_animation_frame()
    		    		.expect("should register `requestAnimationFrame` OK");
					self.animation_id = Some(animation_id);
				},
				None => self.animation_id = None
    	    }
		} else {
			self.animation_id = None;
		}
	}
	
	fn reset_timer(& mut self) {
		match &self.welf {
			Some(w) => {
				let welf = w.clone();
				let delay = (1.0 / self.ticks_per_second_input.value_as_number() * 1000.0) as i32;
				let timer = self.window.set_interval(Box::new(move || {
						(*(welf.upgrade().unwrap().borrow_mut())).universe.tick();
					}),
					delay)
					.expect("Can not set up a timer");
				self.timer = Some(timer);
			}
			None => {}
		}
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