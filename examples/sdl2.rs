#[cfg(feature = "sdl2")]
use gl_object::sdl2_example;

fn main() {
    #[cfg(feature = "sdl2")]
    sdl2_example();
}
