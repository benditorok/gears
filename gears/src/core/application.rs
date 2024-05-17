pub trait EventLoop {}

pub trait GearsApplication {
    async fn run();
}

pub struct Application {}
