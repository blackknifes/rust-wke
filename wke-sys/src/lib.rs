#![allow(warnings)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "enable_report")]
include!(concat!(env!("OUT_DIR"), "/report_bindings.rs"));