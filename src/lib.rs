#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod db;
pub use app::TemplateApp;
pub use db::*;
pub mod utility;
pub mod erust;