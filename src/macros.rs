#[macro_export]
macro_rules! impl_keyboard {
    ($type:ty, $ret:ty, $alias:ident) => {
        impl $type {
            pub fn rows(&self) -> impl Iterator<Item = &Vec<$ret>> {
                self.0.iter()
            }
            pub fn keys(&self) -> impl Iterator<Item = &$ret> {
                self.rows().flatten()
            }
            pub fn shape(&self) -> Shape {
                self.rows().map(|r| r.len()).collect::<Vec<_>>().into()
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
                    .map(|s| s.parse::<$ret>())
                    .collect::<Result<Vec<_>, crate::definitions::DefinitionError>>()
            }
        );
    };
}