#[cfg(not(any(feature = "equation", feature = "system")))]
compile_error!("you have to select exactly one feature between `equation` and `system`");

#[cfg(all(feature = "equation", feature = "system"))]
compile_error!("features `equation` and `system` are mutually exclusive");

fn main() {}
