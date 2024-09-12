macro_rules! gears_collect_strings {
    ($($input:expr),*) => {
        {
            let inputs: &[&str] = &[$($input),*];
            inputs
        }
    };
}
