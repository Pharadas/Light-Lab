#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod gpu_hash;
mod camera;
mod rasterizer;
mod menus;
mod tests;

pub use app::MainApp;
