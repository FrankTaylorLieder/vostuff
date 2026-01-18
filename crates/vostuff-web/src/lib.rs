pub mod app;
pub mod components;
pub mod pages;
pub mod server_fns;

use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

pub use app::App;

// Client-side hydration entry point
#[cfg(feature = "hydrate")]
#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
