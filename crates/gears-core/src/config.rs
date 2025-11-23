/// Configuration options for the application.
pub struct Config {
    /// The title of the window.
    pub window_title: &'static str,
    /// Whether the window should be maximized at startup.
    pub maximized: bool,
    /// Whether to enable debug mode.
    pub debug: bool,
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
            debug: true,
        }
    }
}

impl Config {
    /// Sets the window title.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to set for the window.
    ///
    /// # Returns
    ///
    /// The updated [`Config`] instance.
    pub fn with_window_title(mut self, title: &'static str) -> Self {
        self.window_title = title;
        self
    }

    /// Sets whether the window should be maximized at startup.
    ///
    /// # Arguments
    ///
    /// * `maximized` - A boolean indicating if the window should be maximized.
    ///
    /// # Returns
    ///
    /// The updated [`Config`] instance.
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// Sets whether to enable debug mode.
    ///
    /// # Arguments
    ///
    /// * `debug` - A boolean indicating if debug mode should be enabled.
    ///
    /// # Returns
    ///
    /// The updated [`Config`] instance.
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}
