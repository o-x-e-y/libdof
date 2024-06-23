//! Contains a macro that's used internally to generate a `serde_conv` implementation
//! for `[Fingering](crate::Fingering)` and [`Layer`](crate::Layer), but can be used for anything
//! that would also want to implement [`Keyboard`](crate::Keyboard).

#[macro_export]
/// **NOTE: Depends on [`serde_with`](https://crates.io/crates/serde_with).**
///
/// Macro to generate a `serde_conv` implementation for anything that would also implement
/// [`Keyboard`](crate::Keyboard).
macro_rules! keyboard_conv {
    ($type:ty, $ret:ty, $alias:ident) => {
        serde_with::serde_conv!(
            $alias,
            Vec<$ret>,
            |row: &Vec<$ret>| {
                if row.len() == 0 {
                    ::std::string::String::new()
                } else {
                    row.into_iter()
                        .take(row.len() - 1)
                        .map(|e| format!("{e} "))
                        .chain([row.last().unwrap().to_string()])
                        .collect::<::std::string::String>()
                }
            },
            |line: ::std::string::String| {
                line.split_whitespace()
                    .map(|s| s.parse::<$ret>())
                    .collect::<::std::result::Result<Vec<_>, _>>()
            }
        );
    };
}
