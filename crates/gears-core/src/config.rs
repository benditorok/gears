pub struct Config {
    // Configuration options can be added here as needed
    pub window_title: &'static str,
    pub maximized: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            window_title: "Gears App",
            maximized: false,
        }
    }
}
