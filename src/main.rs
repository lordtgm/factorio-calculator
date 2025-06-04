#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(tuple_trait)]
#![feature(if_let_guard)]
#![feature(try_blocks)]

mod ui;
mod data;
mod model;

use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "full");
    }
    Ok(crate::ui::main()?)
}
