mod notify_test;
mod range_rle_test;
mod string_fuzzy;
use ctor::ctor;

#[ctor]
fn init_color_backtrace() {
    color_backtrace::install();
}