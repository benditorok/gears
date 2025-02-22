pub struct Config {
    pub threadpool_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config { threadpool_size: 8 }
    }
}
