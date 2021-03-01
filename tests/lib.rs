#[cfg(any(not(feature = "debug"), not(feature = "compare")))]
compile_error!(
    "Please enable the `debug` and `compare` features in order to compile and run the tests.
     Example: `cargo test --features debug --features compare`"
);

#[cfg(feature = "abomonation-serialize")]
extern crate abomonation;
#[macro_use]
extern crate approx;
#[cfg(feature = "mint")]
extern crate mint;
extern crate nalgebra as na;
extern crate num_traits as num;

mod core;
mod geometry;
mod linalg;

#[cfg(feature = "proptest-support")]
mod proptest;

//#[cfg(feature = "sparse")]
//mod sparse;
