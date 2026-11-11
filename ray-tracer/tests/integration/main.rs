//! Integration tests: public-API behaviour that spans several modules (see AGENTS.md, "Tests").
//! Everything lives in this one test binary to keep compile and link times down.

mod golden;
mod invariants;
mod output;
mod rendering;
mod support;
mod trait_objects;
mod world;
