/// Configuration options for the application.
pub struct Config {
    /// The title of the window.
    pub window_title: &'static str,
    /// Whether the window should be maximized at startup.
    pub maximized: bool,
    // * Configuration options can be added here as needed
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
