#[macro_use]
extern crate cfg_if;

extern crate wasm_bindgen;
extern crate web_sys;

use wasm_bindgen::prelude::*;

#[macro_use]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
#[macro_use]
macro_rules! error {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into());
    }
}

mod util;


cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        use console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        fn set_panic_hook() {
            error!("panic error!");
        }
    }
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub fn dash_media_url_from_position(document: &str, position: f32, mime_type: &str, role: &str, bandwidth: u32) -> wasm_bindgen::JsValue {
    log!("Position: {}", position);
    let doc = roxmltree::Document::parse(document).unwrap();
    // Make sure that the requested position is not greater than the total duration of the asset or less than 0
    if !util::Dash::requested_position_is_valid(&doc.root_element(), position) || position < 0f32 {
        error!("Requested position {} is out of range", {position});
        return wasm_bindgen::JsValue::undefined();
    }
    let period = util::Dash::find_period(&doc.root_element(), position);
    let adaptation_set = util::Dash::find_adaptation_set(&period, &mime_type, &role);
    if adaptation_set == None {
        error!("No AdaptationSet found for mimeType {} and role {}", mime_type, role);
        return wasm_bindgen::JsValue::undefined();
    }

    let adaptation_set = adaptation_set.unwrap();
    let media_url = util::Dash::get_media_from_adaptation_set(&adaptation_set, bandwidth, position);
    if media_url == None {
        error!("FAIL");
        return wasm_bindgen::JsValue::undefined()
    }
    return wasm_bindgen::JsValue::from_str(&media_url.unwrap());
}

