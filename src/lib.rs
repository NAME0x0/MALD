#![allow(dead_code)]

pub mod ai;
pub mod cli;
pub mod commands;
pub mod config;
pub mod daemon;
pub mod errors;
pub mod fs;
#[cfg(feature = "gui")]
pub mod gui;
pub mod index;
pub mod parser;
pub mod web_assets;
