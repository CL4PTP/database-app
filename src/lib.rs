#![feature(self_struct_ctor)]
#![feature(nll)]
#![feature(type_ascription)]

// TODO: use bitsets instead of actual sets
// extern crate fixedbitset;

extern crate cfg_if;
extern crate wasm_bindgen;
extern crate web_sys;

#[macro_use]
mod utils;

pub mod functional_dependencies;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

use functional_dependencies::*;

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
pub fn candidate_keys(s: &str) -> Option<String> {
    let fd: DependencySet = if let Ok(fd) = s.parse() { fd } else { return None };

    Some(
        fd.candidate_keys(&fd.effective_attributes())
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}
