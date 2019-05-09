//! # RIV
//!
//! The riv library exposes all the functionality required to run the riv command line image viewer
//! app

#![deny(
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_must_use
)]
pub mod cli;
pub mod paths;
pub mod program;
pub mod screen;
pub mod ui;
