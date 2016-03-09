extern crate glob;
extern crate syntex;
extern crate serde_codegen;

use std::path::Path;
use glob::glob;
use syntex::Registry;

pub fn main() {
  for entry in glob("src/**/*.gen.rs").unwrap() {
    let src = entry.unwrap();
    let dst = src.to_str().unwrap().replace(".gen.rs", ".rs");
    let mut registry = Registry::new();
    serde_codegen::register(&mut registry);
    registry.expand("", &src, Path::new(&dst)).unwrap();
  }
}
