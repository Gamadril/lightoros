use std::ffi::c_void;
//use std::thread;
//use std::time::Duration;

use lightoros_plugin::{PluginInfo, PluginInputTrait, TraitData, RGB};

use serde::Deserialize;

type CGError = i32;
#[allow(non_upper_case_globals)]
const kCGErrorSuccess: CGError = 0;

type CGImageRef = *mut u8;
type CGDataProviderRef = *mut u8;
type CFDataRef = *const u8;

type CGDirectDisplayID = u32;
type CGDisplayCount = u32;

#[derive(Deserialize, Debug)]
struct Config {
    screen_index: u32,
    delay_frame: u64
}

struct OsxScreenGrabberInput {
    config: Config,
    display: CGDirectDisplayID,
}

impl OsxScreenGrabberInput {
    fn new(config: &serde_json::Value) -> OsxScreenGrabberInput {
        let cfg = config.clone();
        let config = match serde_json::from_value(cfg) {
            Ok(config) => config,
            Err(err) => {
                panic!("Error deserializing configuration: {}", err);
            }
        };

        OsxScreenGrabberInput {
            config: config,
            display: unsafe { CGMainDisplayID() },
        }
    }
}

impl PluginInputTrait for OsxScreenGrabberInput {
    fn init(&mut self) -> bool {
        const MAX_DISPLAYS: u32 = 8;
        let mut count: CGDisplayCount = 0;

        let mut displays: Vec<CGDirectDisplayID> = vec![0; MAX_DISPLAYS as usize];
        let err = unsafe {
            CGGetActiveDisplayList(
                MAX_DISPLAYS,
                &mut displays[0] as *mut CGDirectDisplayID,
                &mut count,
            )
        };

        if err != kCGErrorSuccess {
            eprintln!("Error getting list of displays");
            return false;
        }

        if self.config.screen_index > count - 1 {
            eprintln!(
                "Display with index {} is not available. Using main display.",
                self.config.screen_index
            );
        } else {
            self.display = displays[self.config.screen_index as usize];
        }
        true
    }

    fn get(&mut self) -> Option<TraitData> {
        let mut disp_image = unsafe { CGDisplayCreateImage(self.display) };

        // display probably lost, use main display
        if disp_image.is_null() {
            disp_image = unsafe { CGDisplayCreateImage(CGMainDisplayID()) };
            // no displays connected, panic
            if disp_image.is_null() {
                panic!("No display connected.");
            }
        }

        let width: usize = unsafe { CGImageGetWidth(disp_image) };
        let height: usize = unsafe { CGImageGetHeight(disp_image) };

        let image_data = unsafe { CGDataProviderCopyData(CGImageGetDataProvider(disp_image)) };
        let p_data = unsafe { CFDataGetBytePtr(image_data) };
        let data_len = unsafe { CFDataGetLength(image_data) };

        let raw_data = unsafe { std::slice::from_raw_parts(p_data, data_len as usize) }.to_vec();
        let size = width * height;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);
        for i in 0..size {
            data_out.push(RGB {
                r: raw_data[i * 4 + 2],
                g: raw_data[i * 4 + 1],
                b: raw_data[i * 4],
            });
        }

        unsafe {
            CFRelease(image_data as *const c_void);
            CGImageRelease(disp_image);
        }

        std::thread::sleep(std::time::Duration::from_millis(self.config.delay_frame));

        let result: TraitData = TraitData {
            rgb: data_out,
            meta: [
                ("width".to_string(), width.to_string()),
                ("height".to_string(), height.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        };
        Some(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> Box<dyn PluginInputTrait> {
    let plugin = OsxScreenGrabberInput::new(config);
    Box::new(plugin)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    PluginInfo {
        api_version: 1,
        name: "OsxScreenGrabberInput",
        filename: env!("CARGO_PKG_NAME"),
    }
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
    fn CFDataGetBytePtr(theData: CFDataRef) -> *const u8;
    fn CFDataGetLength(theData: CFDataRef) -> i64;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGDisplayCreateImage(displayID: CGDirectDisplayID) -> CGImageRef;
    fn CGMainDisplayID() -> u32;
    fn CGGetActiveDisplayList(
        max_displays: u32,
        active_displays: *mut CGDirectDisplayID,
        display_count: *mut u32,
    ) -> CGError;
    fn CGImageRelease(image: CGImageRef);

    fn CGImageGetDataProvider(image: CGImageRef) -> CGDataProviderRef;
    fn CGImageGetHeight(image: CGImageRef) -> usize;
    fn CGImageGetWidth(image: CGImageRef) -> usize;

    fn CGDataProviderCopyData(provider: CGDataProviderRef) -> CFDataRef;
}
