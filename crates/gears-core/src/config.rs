/// Configuration options for the application.
pub struct Config {
    /// The title of the window.
    pub window_title: &'static str,
    /// Whether the window should be maximized at startup.
    pub maximized: bool,
    /// The width of the window.
    pub window_width: u32,
    /// The height of the window.
    pub window_height: u32,
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
            window_width: 1280,
            window_height: 720,
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

    /// Sets the window size.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the window.
    /// * `height` - The height of the window.
    ///
    /// # Returns
    ///
    /// The updated [`Config`] instance.
    pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
        self.window_width = width;
        self.window_height = height;
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
