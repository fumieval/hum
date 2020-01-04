extern crate vst;
extern crate winit;

use std::env;
use std::error::Error;
use std::path::Path;
use std::process;
use std::sync::{Arc, Mutex};
use std::ffi;

use vst::host::{Host, PluginLoader};
use vst::plugin::Plugin;
use winit::os::macos::{WindowExt};

#[allow(dead_code)]
struct SampleHost;

impl Host for SampleHost {
    fn automate(&self, index: i32, value: f32) {
        println!("Parameter {} had its value changed to {}", index, value);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: simple_host path/to/vst");
        process::exit(1);
    }

    let path = Path::new(&args[1]);

    // Create the host
    let host = Arc::new(Mutex::new(SampleHost));

    println!("Loading {}...", path.to_str().unwrap());

    // Load the plugin
    let mut loader = PluginLoader::load(path, Arc::clone(&host))
        .unwrap_or_else(|e| panic!("Failed to load plugin: {}", e.description()));

    // Create an instance of the plugin
    let mut instance = loader.instance().unwrap();

    // Get the plugin information
    let info = instance.get_info();

    println!(
        "Loaded '{}':\n\t\
         Vendor: {}\n\t\
         Presets: {}\n\t\
         Parameters: {}\n\t\
         VST ID: {}\n\t\
         Version: {}\n\t\
         Initial Delay: {} samples",
        info.name, info.vendor, info.presets, info.parameters, info.unique_id, info.version, info.initial_delay
    );

    // Initialize the instance
    instance.init();
    println!("Initialized instance!");

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::Window::new(&events_loop).unwrap();

    let parent : *mut ffi::c_void = window.get_nsview();

    match instance.get_editor()
    {
        Some(ref mut mutex_editor) => {
            let mut editor = mutex_editor.lock().unwrap();
            let (width, height) = editor.size();
            window.set_inner_size(winit::dpi::LogicalSize { width: width as f64, height: height as f64 });
            if editor.open(parent)
            {
                println!("Opening an editor for {}", info.name);
            }
            else
            {
                println!("Failed to open an editor for {}", info.name)
            }
        }
        None => println!("Editor not found")
    }
    events_loop.run_forever(|event| {
        match event {
            winit::Event::WindowEvent {
              event: winit::WindowEvent::CloseRequested,
              ..
            } => winit::ControlFlow::Break,
            _ => winit::ControlFlow::Continue,
        }
    });
}
