pub trait Gui {
    fn new() -> Self;
    fn show(&mut self);
    fn hide(&mut self);
}
