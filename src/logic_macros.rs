// Licensed: Apache 2.0
// (c) Campbell Barton

// Basic Logic
//
// Statements we can't easily express in Rust.
//

/// Check if the first argument in any of the following arguments, eg:
///
/// ```.text
/// if elem!(my_var, FOO, BAR, BAZ) { ... }
/// ```
///
#[macro_export]
macro_rules! elem {
    ($val:expr, $($var:expr), *) => {
        $($val == $var) || *
    }
}
