use pipewire::{MainLoop, Context};
use pipewire::types::ObjectType;

fn main() {
    let mainloop = MainLoop::new().unwrap();
    let context = Context::new(&mainloop).unwrap();
    let core = context.connect(None).unwrap();
    let registry = core.get_registry().unwrap();

    let _listener = registry.add_listener_local().global(|global| {
        if global.type_== ObjectType::Device {
            dbg!(global);
        }
    }).register();

    mainloop.run();
    println!("Hello, world!");
}
