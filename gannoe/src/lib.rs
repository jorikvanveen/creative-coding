use winit::event_loop::EventLoop;
use winit::window::Window;

struct Gannoe {
    event_loop: EventLoop<()>,
    window: Window
}

impl Gannoe {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = Window::new(&event_loop).expect("Failed to create window");
        Self { event_loop, window }
    }

    pub fn run(self) {
        self.event_loop.run(|_, _, _| {});
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let gannoe = Gannoe::new();
        gannoe.run();
    }
}
