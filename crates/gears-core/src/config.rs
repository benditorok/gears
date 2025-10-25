/// Configuration options for the application.
pub struct Config {
    // Configuration options can be added here as needed
    pub window_title: &'static str,
    pub maximized: bool,
}

impl Default for Config {
    /// Creates the default configuration for the application.
    ///
    /// # Returns
    ///
    /// The default [`Config`] instance.
    fn default() -> Self {
        Config {
            window_title: "Gears App",
            maximized: false,
        }
    }
}
