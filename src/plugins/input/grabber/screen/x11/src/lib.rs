#![cfg(target_os = "linux")]

use lightoros_plugin_base::input::{CreateInputPluginResult, PluginInputTrait};
use lightoros_plugin_base::*;

use std::ffi::c_void;
use std::mem;
use std::os::raw::{c_char, c_int, c_long, c_uint, c_ulong};
use std::ptr;

use serde::Deserialize;

const NAME: &str = "X11ScreenGrabberInput";

#[derive(Deserialize, Debug)]
struct Config {
    delay_frame: u64,
}

struct X11ScreenGrabberInput {
    config: Config,
    logger: Logger,
}

impl X11ScreenGrabberInput {
    fn create(config: &serde_json::Value) -> CreateInputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = X11ScreenGrabberInput {
            config: config,
            logger: Logger::new(NAME.to_string()),
        };

        Ok(Box::new(plugin))
    }
}

impl std::fmt::Display for X11ScreenGrabberInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl PluginInputTrait for X11ScreenGrabberInput {
    fn init(&mut self) -> PluginResult<()> {
        unsafe {
            let display = XOpenDisplay(ptr::null());

            if display.is_null() {
                return plugin_err!("Error opening display with XOpenDisplay.");
            }
            let screen = XDefaultScreen(display);
            let root = XRootWindow(display, screen);
            let mut attrs = mem::MaybeUninit::uninit();
            XGetWindowAttributes(display, root, attrs.as_mut_ptr());
            let attrs: XWindowAttributes = attrs.assume_init();

            self.logger.debug(&format!(
                "Opened X11 window with resolution: {}x{}@{}bit",
                attrs.width, attrs.height, attrs.depth
            ));
            XCloseDisplay(display);
        }

        Ok(())
    }

    fn get(&mut self) -> PluginResult<TraitData> {
        let width: c_int;
        let height: c_int;

        let display = unsafe { XOpenDisplay(ptr::null()) };

        if display.is_null() {
            return plugin_err!("Error opening display with XOpenDisplay.");
        }

        let screen = unsafe { XDefaultScreen(display) };
        let root = unsafe { XRootWindow(display, screen) };

        unsafe {
            let mut attrs = mem::MaybeUninit::uninit();
            XGetWindowAttributes(display, root, attrs.as_mut_ptr());
            let xinfo = attrs.assume_init();
            width = xinfo.width;
            height = xinfo.height;
        };

        let ximage = unsafe {
            XGetImage(
                display,
                root,
                0,
                0,
                width as c_uint,
                height as c_uint,
                XAllPlanes(),
                ZPixmap,
            )
        };
        
        let image = {
            if ximage.is_null() {
                return plugin_err!("Error getting image.");
            } else {
                unsafe { &*ximage }
            }
        };

        let size = width * height;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel = unsafe { XGetPixel(ximage, x, y) };
                let b = (pixel & image.blue_mask) as u8;
                let g = ((pixel & image.green_mask) >> 8) as u8;
                let r = ((pixel & image.red_mask) >> 16) as u8;
                data_out.push(RGB { b: b, g: g, r: r });
            }
        }

        unsafe { 
            XDestroyImage(ximage);
            XCloseDisplay(display);
        }

        std::thread::sleep(std::time::Duration::from_millis(self.config.delay_frame));

        let result = plugin_data!(data_out, {
            "width" => width,
            "height" => height,
        });
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateInputPluginResult {
    X11ScreenGrabberInput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Input)
}

#[allow(non_upper_case_globals)]
static ZPixmap: c_int = 2;

type Display = c_void;
type Window = c_ulong;
type VisualID = c_ulong;
type Colormap = c_ulong;
type XPointer = *mut c_char;
type Drawable = c_ulong;

#[repr(C)]
struct Depth {
    pub depth: c_int,
    pub nvisuals: c_int,
    pub visuals: *mut Visual,
}

#[repr(C)]
struct XExtData {
    pub number: c_int,
    pub next: *mut c_void,
    pub free_private: *mut u8,
    pub private_data: XPointer,
}

#[repr(C)]
struct Visual {
    pub ext_data: *mut XExtData,
    pub visualid: VisualID,
    pub _class: c_int,
    pub red_mask: c_ulong,
    pub green_mask: c_ulong,
    pub blue_mask: c_ulong,
    pub bits_per_rgb: c_int,
    pub map_entries: c_int,
}

#[repr(C)]
struct Screen {
    pub ext_data: *mut XExtData,
    pub display: *mut c_void,
    pub root: Window,
    pub width: c_int,
    pub height: c_int,
    pub mwidth: c_int,
    pub mheight: c_int,
    pub ndepths: c_int,
    pub depths: *mut Depth,
    pub root_depth: c_int,
    pub root_visual: *mut Visual,
    pub default_gc: *mut c_void,
    pub cmap: Colormap,
    pub white_pixel: c_ulong,
    pub black_pixel: c_ulong,
    pub max_maps: c_int,
    pub min_maps: c_int,
    pub backing_store: c_int,
    pub save_unders: c_int,
    pub root_input_mask: c_long,
}

#[repr(C)]
struct XWindowAttributes {
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub border_width: c_int,
    pub depth: c_int,
    pub visual: *mut Visual,
    pub root: Window,
    pub _class: c_int,
    pub bit_gravity: c_int,
    pub win_gravity: c_int,
    pub backing_store: c_int,
    pub backing_planes: c_ulong,
    pub backing_pixel: c_ulong,
    pub save_under: c_int,
    pub colormap: Colormap,
    pub map_installed: c_int,
    pub map_state: c_int,
    pub all_event_masks: c_long,
    pub your_event_mask: c_long,
    pub do_not_propagate_mask: c_long,
    pub override_redirect: c_int,
    pub screen: *mut Screen,
}

#[repr(C)]
struct XImage {
    pub width: c_int,
    pub height: c_int,
    pub xoffset: c_int,
    pub format: c_int,
    pub data: *mut c_char,
    pub byte_order: c_int,
    pub bitmap_unit: c_int,
    pub bitmap_bit_order: c_int,
    pub bitmap_pad: c_int,
    pub depth: c_int,
    pub bytes_per_line: c_int,
    pub bits_per_pixel: c_int,
    pub red_mask: c_ulong,
    pub green_mask: c_ulong,
    pub blue_mask: c_ulong,
    pub obdata: XPointer,
    pub f: struct_funcs,
}

#[repr(C)]
struct struct_funcs {
    pub create_image: *mut u8,
    pub destroy_image: *mut u8,
    pub get_pixel: *mut u8,
    pub put_pixel: *mut u8,
    pub sub_image: *mut u8,
    pub add_pixel: *mut u8,
}

#[link(name = "X11")]
extern "C" {
    fn XAllPlanes() -> c_ulong;
    fn XOpenDisplay(display_name: *const c_char) -> *mut Display;
    fn XGetPixel(image: *mut XImage, x: c_int, y: c_int) -> c_ulong;
    fn XCloseDisplay(display: *mut Display) -> c_int;
    fn XDefaultScreen(display: *mut Display) -> c_int;
    fn XRootWindow(display: *mut Display, screen_number: c_int) -> Window;
    fn XGetWindowAttributes(
        display: *mut Display,
        window: Window,
        window_attributes: *mut XWindowAttributes,
    ) -> c_int;
    fn XGetImage(
        display: *mut Display,
        d: Drawable,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        plane_mask: c_ulong,
        format: c_int,
    ) -> *mut XImage;
    fn XDestroyImage(image: *mut XImage) -> c_int;
}
