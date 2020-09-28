use wasm_bindgen::JsCast;

use std::cell::RefCell;
use std::rc::Rc;

extern crate web_sys;
use web_sys::Element;
use web_sys::Event;
use web_sys::EventTarget;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlElement;
use web_sys::Window;
use wasm_bindgen::prelude::*;

pub struct RegisteredHtmlEvent<'a> {
	#[allow(dead_code)]
	event_target: EventTarget,
	type_: &'a str,
	
	#[allow(dead_code)]
	closure: wasm_bindgen::closure::Closure<dyn std::ops::FnMut(Event)>
}

pub struct HtmlEvents<'a> {
	event_target: &'a EventTarget
}

impl HtmlEvents<'_> {
	#[allow(dead_code)]
	pub fn add_event_listener<'a>(&self, type_: &'a str, listener: Box<dyn Fn(Event) -> ()>) -> Result<RegisteredHtmlEvent<'a>, JsValue> {
	
   		let closure = Closure::wrap(Box::new(move |event| listener(event)) as Box<dyn FnMut(Event)>);
   	
    	match self.event_target.add_event_listener_with_callback(type_, closure.as_ref().unchecked_ref()) {
			Err(e) => Err(e),
		    _ => {
				Ok(RegisteredHtmlEvent {
					event_target: self.event_target.clone(),
					type_,
					closure: closure
				})
		    }
		}
	}

	#[allow(dead_code)]
	pub fn add_event_listener_state<'a, T: 'static>(&self, type_: &'a str, state: T, listener: Box<dyn Fn(&T, Event) -> ()>) -> Result<RegisteredHtmlEvent<'a>, JsValue> {
		
   		let closure = Closure::wrap(Box::new(move |event| {
   				listener(&state, event);
   		}) as Box<dyn FnMut(Event)>);
   	
    	match self.event_target.add_event_listener_with_callback(type_, closure.as_ref().unchecked_ref()) {
			Err(e) => Err(e),
		    _ => {
				Ok(RegisteredHtmlEvent {
					event_target: self.event_target.clone(),
					type_,
					closure
				})
		    }
		}
	}

/*
	#[allow(dead_code)]
	pub fn add_event_listener_borrow<'a, T>(&self, type_: &'a str, mut state: &'static T, listener: Box<dyn Fn(&T, Event) -> ()>) -> Result<RegisteredHtmlEvent<'a>, JsValue> {
		
   		let closure = Closure::wrap(Box::new(move |event| {
   				listener(state, event);
   		}) as Box<dyn FnMut(Event)>);
   	
    	match self.event_target.add_event_listener_with_callback(type_, closure.as_ref().unchecked_ref()) {
			Err(e) => Err(e),
		    _ => {
				Ok(RegisteredHtmlEvent {
					event_target: self.event_target.clone(),
					type_,
					closure
				})
		    }
		}
	}*/
}

impl<'a> Drop for RegisteredHtmlEvent<'a> {
	fn drop(&mut self) {
		self.event_target.remove_event_listener_with_callback(self.type_, self.closure.as_ref().unchecked_ref()).unwrap();
	}
}

// Allow easy access to the HtmlEvents API

pub trait HtmlExt {
	fn events(&self) -> HtmlEvents;
}

impl HtmlExt for HtmlElement {
	fn events(&self) -> HtmlEvents {
		HtmlEvents {
			event_target: self.dyn_ref::<EventTarget>().unwrap(),
		}
	}
}

impl HtmlExt for HtmlCanvasElement {
	fn events(&self) -> HtmlEvents {
		HtmlEvents {
			event_target: self.dyn_ref::<EventTarget>().unwrap(),
		}
	}
}

impl HtmlExt for Element {
	fn events(&self) -> HtmlEvents {
		HtmlEvents {
			event_target: self.dyn_ref::<EventTarget>().unwrap(),
		}
	}
}

// Simplified use of timers

pub struct IntervalSubscription {
	window: Window,
	timer: i32,

	#[allow(dead_code)]
	closure: wasm_bindgen::closure::Closure<dyn std::ops::FnMut()>,
}

// Simplified use of animation frame callbacks
pub struct AnimationFrameRequester {
	#[allow(dead_code)]
	window: Window,

	#[allow(dead_code)]
	render_loop_closure: Rc<RefCell<Option<Closure<dyn std::ops::FnMut(i32)>>>>,
}

pub trait WindowExt {
	fn set_interval(&self, handler: Box<dyn Fn() -> ()>, delay: i32) -> Result<IntervalSubscription, JsValue>;
	fn prepare_animation_frame_callback(&self, handler: Box<dyn Fn(i32) -> ()>) -> AnimationFrameRequester;
}

impl WindowExt for Window {
	fn set_interval(&self, handler: Box<dyn Fn() -> ()>, delay: i32) -> Result<IntervalSubscription, JsValue> {
   		let closure = Closure::wrap(Box::new(move || handler()) as Box<dyn FnMut()>);
		match self.set_interval_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), delay) {
			Err(e) => Err(e),
			Ok(timer) => Ok(IntervalSubscription {
				window: self.clone(),
				timer,
				closure		
			})
		}
	}

	fn prepare_animation_frame_callback(&self, handler: Box<dyn Fn(i32) -> ()>) -> AnimationFrameRequester {
    	// TODO: Hack for requestAnimationFrame loop
		// https://github.com/bzar/wasm-pong-rs/blob/master/src/lib.rs
	    let render_loop_closure = Rc::new(RefCell::new(None));
    	let g = render_loop_closure.clone();
    	*g.borrow_mut() = Some(Closure::wrap(Box::new(move |animation_id| handler(animation_id)) as Box<dyn FnMut(i32)>));    	

		AnimationFrameRequester {
			window: self.clone(),
			render_loop_closure
		}
	}
}

impl<'a> Drop for IntervalSubscription {
	fn drop(&mut self) {
		self.window.clear_interval_with_handle(self.timer);
	}
}

impl AnimationFrameRequester {
	#[allow(dead_code)]
	pub fn request_animation_frame(&self) -> Result<i32, JsValue> {
    	self.window.request_animation_frame(
   			self.render_loop_closure.borrow().as_ref().unwrap().as_ref().unchecked_ref())
	}
}