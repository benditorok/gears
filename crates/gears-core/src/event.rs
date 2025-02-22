use std::fmt::Debug;

// #[derive(Debug)]
// pub enum WindowEvent {
//     Resize(u32, u32), // width, height
//     Update,
//     Redraw,
// }

// #[derive(Debug)]
// pub enum DeviceEvent {
//     MouseMotion,
//     MouseWheel,
//     KeyboardInput,
// }

#[derive(Debug)]
pub enum GearsEvent {
    Shoot { pos: [i32; 3], ray: [i32; 3] },
}

// pub struct EventSystem {
//     sender: Sender<GearsEvent>,
//     receiver: Receiver<GearsEvent>,
// }

// impl Default for EventSystem {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl EventSystem {
//     pub fn new() -> Self {
//         let (sender, receiver) = mpsc::channel();
//         Self { sender, receiver }
//     }

//     pub(crate) fn try_receive(&self) -> Option<GearsEvent> {
//         match self.receiver.try_recv() {
//             Ok(event) => Some(event),
//             Err(TryRecvError::Disconnected) => {
//                 error!("Failed to receive event: channel disconnected");
//                 None
//             }
//             Err(TryRecvError::Empty) => None,
//         }
//     }

//     pub fn send(&self, event: GearsEvent) -> bool {
//         if let Err(SendError(e)) = self.sender.send(event) {
//             error!("Failed to send event: channel disconnected. Event: {:?}", e);
//             false
//         } else {
//             true
//         }
//     }
// }
