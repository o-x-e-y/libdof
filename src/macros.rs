//! Contains a macro that's used internally to generate the same code for both `Fingering` and `Layer`.

#[macro_export]
/// Macro for implementing functionality for `Fingering` and `Layer`
macro_rules! impl_keyboard {
    ($type:ty, $ret:ty, $alias:ident) => {
        impl $type {
            /// Get an iterator over each row.
            pub fn rows(&self) -> impl Iterator<Item = &Vec<$ret>> {
                self.0.iter()
            }
            /// Get an iterator over the individual keys.
            pub fn keys(&self) -> impl Iterator<Item = &$ret> {
                self.rows().flatten()
            }
            /// Get the shape.
            pub fn shape(&self) -> Shape {
                self.rows().map(|r| r.len()).collect::<Vec<_>>().into()
            }
            /// Get the amount of rows.
            pub fn row_count(&self) -> usize {
                self.0.len()
            }
            /// Get a reference to the inner rows.
            pub fn inner(&self) -> &Vec<Vec<$ret>> {
                &self.0
            }
            /// Convert into inner vecs.
            pub fn into_inner(self) -> Vec<Vec<$ret>> {
                self.0
            }
        }

        impl From<Vec<Vec<$ret>>> for $type {
            fn from(f: Vec<Vec<$ret>>) -> Self {
                Self(f)
            }
        }

        serde_conv!(
            $alias,
            Vec<$ret>,
            |row: &Vec<$ret>| {
                if row.len() == 0 {
                    String::new()
                } else {
                    row.into_iter()
                        .take(row.len() - 1)
                        .map(|e| format!("{e} "))
                        .chain([row.last().unwrap().to_string()])
                        .collect::<String>()
                }
            },
            |line: String| {
                line.split_whitespace()
                    .map(|s| s.parse::<$ret>().map_err(Into::into))
                    .collect::<Result<Vec<_>, $crate::dofinitions::DofinitionError>>()
            }
        );
    };
}
