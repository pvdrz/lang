#[macro_export]
macro_rules! def_gen {
    ($name:ident => $newtype:ident) => {
        pub(crate) struct $name {
            count: usize,
        }

        impl $name {
            pub(crate) fn new() -> Self {
                Self { count: 0 }
            }

            pub(crate) fn generate(&mut self) -> $newtype {
                let generated = $newtype(self.count);
                self.count += 1;
                generated
            }
        }
    };
}

