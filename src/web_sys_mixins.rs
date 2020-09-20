use wasm_bindgen::JsCast;

use std::mem;

extern crate web_sys;
use web_sys::Element;
use web_sys::Event;
use web_sys::EventTarget;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlElement;
use wasm_bindgen::prelude::*;

pub struct RegisteredHtmlEvent<'a> {
	#[allow(dead_code)]
	event_target: EventTarget,
	type_: &'a str,
	closure: Option<wasm_bindgen::closure::Closure<dyn std::ops::FnMut(Event)>>
}

impl<'a> RegisteredHtmlEvent<'a> {
	#[allow(dead_code)]
	pub fn forget(mut self) {
		if self.closure.is_some() {
			let closure = mem::replace(&mut self.closure, None);
			closure.unwrap().forget();
		}
	}
}

impl<'a> Drop for RegisteredHtmlEvent<'a> {
	fn drop(&mut self) {
		match &self.closure {
			Some(closure) => {
				self.event_target.remove_event_listener_with_callback(self.type_, closure.as_ref().unchecked_ref()).unwrap();
			},
			None => {}
		};
	}
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
					closure: Some(closure)
				})
		    }
		}
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
