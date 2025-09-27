pub struct Config {
    // Configuration options can be added here as needed
    pub window_title: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            window_title: "Gears App",
        }
    }
}
