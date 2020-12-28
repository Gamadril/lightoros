#![cfg(target_os = "linux")]

use lightoros_plugin_base::input::{CreateInputPluginResult, PluginInputTrait};
use lightoros_plugin_base::*;

use std::os::unix::io::IntoRawFd;

use serde::Deserialize;
use std::fs::OpenOptions;

const NAME: &str = "FramebufferScreenGrabberInput";

const FBIOGET_VSCREENINFO: u64 = 0x4600;
const FBIOGET_FSCREENINFO: u64 = 0x4602;
const PROT_READ: i32 = 1;
const MAP_PRIVATE: i32 = 2;

#[derive(Deserialize, Debug)]
struct Config {
    path: String,
    delay_frame: u64,
}

struct FramebufferScreenGrabberInput {
    config: Config,
    logger: Logger,
}

impl FramebufferScreenGrabberInput {
    fn create(config: &serde_json::Value) -> CreateInputPluginResult {
        let config = plugin_config_or_return!(config.clone());

        let plugin = FramebufferScreenGrabberInput {
            config: config,
            logger: Logger::new(NAME.to_string()),
        };

        Ok(Box::new(plugin))
    }
}

impl std::fmt::Display for FramebufferScreenGrabberInput {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        NAME.fmt(f)
    }
}

impl PluginInputTrait for FramebufferScreenGrabberInput {
    fn init(&mut self) -> PluginResult<()> {
        let fb = match OpenOptions::new().read(true).open(&self.config.path) {
            Ok(fb) => fb,
            Err(err) => {
                return plugin_err!(
                    "Error opening the framebuffer {}: {}",
                    self.config.path,
                    err
                );
            }
        };
        let fd = fb.into_raw_fd();

        let mut vinfo = fb_var_screeninfo::default();
        let result = unsafe { ioctl(fd, FBIOGET_VSCREENINFO, &mut vinfo) };

        if result != 0 {
            return plugin_err!("Could not get screen information");
        }

        self.logger.debug(&format!(
            "Opened framebuffer '{}' with resolution: {}x{}@{}bit",
            self.config.path, vinfo.xres, vinfo.yres, vinfo.bits_per_pixel
        ));

        if vinfo.bits_per_pixel == 32 {
            let mut order: [char; 3] = [' '; 3];
            order[(vinfo.red.offset / 8) as usize] = 'R';
            order[(vinfo.green.offset / 8) as usize] = 'G';
            order[(vinfo.blue.offset / 8) as usize] = 'B';
            self.logger.debug(&format!(
                "Color order: {}",
                order.iter().collect::<String>()
            ));
        }

        Ok(())
    }

    fn get(&mut self) -> PluginResult<TraitData> {
        let fb = match OpenOptions::new().read(true).open(&self.config.path) {
            Ok(fb) => fb,
            Err(err) => {
                return plugin_err!(
                    "Error opening the framebuffer {}: {}",
                    self.config.path,
                    err
                );
            }
        };

        let mut vinfo = fb_var_screeninfo::default();
        let fd = fb.into_raw_fd();

        let mut result = unsafe { ioctl(fd, FBIOGET_VSCREENINFO, &mut vinfo) };
        if result != 0 {
            return plugin_err!("Could not get screen information");
        }

        let mut finfo = fb_fix_screeninfo::default();
        result = unsafe { ioctl(fd, FBIOGET_FSCREENINFO, &mut finfo) };
        if result != 0 {
            return plugin_err!("Could not get screen information");
        }

        let bytes_per_pixel = vinfo.bits_per_pixel / 8;
        let data_size: usize = (finfo.line_length * vinfo.yres) as usize;

        /* map the device to memory */
        let fbp = unsafe { mmap(std::ptr::null(), data_size, PROT_READ, MAP_PRIVATE, fd, 0) };
        let raw_data = unsafe { std::slice::from_raw_parts(fbp, data_size) };

        let size = (vinfo.xres * vinfo.yres) as usize;
        let mut data_out: Vec<RGB> = Vec::with_capacity(size);

        for y in 0..vinfo.yres {
            for x in 0..vinfo.xres {
                let idx = (y * finfo.line_length + x * bytes_per_pixel) as usize;
                data_out.push(RGB {
                    b: raw_data[idx],
                    g: raw_data[idx + 1],
                    r: raw_data[idx + 2],
                });
            }
        }

        unsafe {
            munmap(fbp, data_size);
            close(fd);
        }

        std::thread::sleep(std::time::Duration::from_millis(self.config.delay_frame));

        let result = plugin_data!(data_out, {
            "width" => vinfo.xres,
            "height" => vinfo.yres,
        });
        Ok(result)
    }
}

#[no_mangle]
pub fn create(config: &serde_json::Value) -> CreateInputPluginResult {
    FramebufferScreenGrabberInput::create(config)
}

#[no_mangle]
pub fn info() -> PluginInfo {
    plugin_info!(1, NAME, PluginKind::Input)
}

#[repr(C)]
#[derive(Clone, Default, Debug)]
pub struct fb_bitfield {
    pub offset: u32,    /* beginning of bitfield */
    pub length: u32,    /* length of bitfield */
    pub msb_right: u32, /* != 0 : Most significant bit is right */
}

#[repr(C)]
#[derive(Clone, Default, Debug)]
pub struct fb_fix_screeninfo {
    pub id: [u8; 16],       /* identification string eg "TT Builtin" */
    pub smem_start: u64,    /* Start of frame buffer mem (physical address) */
    pub smem_len: u32,      /* Length of frame buffer mem */
    pub r#type: u32,        /* see FB_TYPE_*                */
    pub type_aux: u32,      /* Interleave for interleaved Planes */
    pub visual: u32,        /* see FB_VISUAL_*              */
    pub xpanstep: u16,      /* zero if no hardware panning  */
    pub ypanstep: u16,      /* zero if no hardware panning  */
    pub ywrapstep: u16,     /* zero if no hardware panning  */
    pub line_length: u32,   /* length of a line in bytes    */
    pub mmio_start: u64,    /* Start of Memory Mapped I/O (physical address)  */
    pub mmio_len: u32,      /* Length of Memory Mapped I/O  */
    pub accel: u32,         /* Indicate to driver which specific chip/card we have    */
    pub capabilities: u16,  /* see FB_CAP_*                 */
    pub reserved: [u16; 2], /* Reserved for future compatibility */
}

#[repr(C)]
#[derive(Clone, Default, Debug)]
pub struct fb_var_screeninfo {
    pub xres: u32, /* visible resolution	*/
    pub yres: u32,
    pub xres_virtual: u32, /* virtual resolution	resolution */
    pub yres_virtual: u32,
    pub xoffset: u32, /* offset from virtual to visible */
    pub yoffset: u32,
    pub bits_per_pixel: u32, /* guess what */
    pub grayscale: u32,      /* 0 = color, 1 = grayscale,	 >1 = FOURCC */
    pub red: fb_bitfield,    /* bitfield in fb mem if true color, else only length is significant */
    pub green: fb_bitfield,
    pub blue: fb_bitfield,
    pub transp: fb_bitfield, /* transparency */
    pub nonstd: u32,         /* != 0 Non standard pixel format */
    pub activate: u32,       /* see FB_ACTIVATE_* */
    pub height: u32,         /* height of picture in mm */
    pub width: u32,          /* width of picture in mm */
    pub accel_flags: u32,    /* (OBSOLETE) see fb_info.flags */
    /* Timing: All values in pixclocks, except pixclock (of course) */
    pub pixclock: u32,     /* pixel clock in ps (pico seconds) */
    pub left_margin: u32,  /* time from sync to picture */
    pub right_margin: u32, /* time from picture to sync */
    pub upper_margin: u32, /* time from sync to picture */
    pub lower_margin: u32,
    pub hsync_len: u32,     /* length of horizontal sync */
    pub vsync_len: u32,     /* length of vertical sync */
    pub sync: u32,          /* see FB_SYNC_* */
    pub vmode: u32,         /* see FB_VMODE_* */
    pub rotate: u32,        /* angle we rotate counter clockwise */
    pub colorspace: u32,    /* colorspace for FOURCC-based modes */
    pub reserved: [u32; 4], /* Reserved for future compatibility */
}

extern "C" {
    pub fn close(fd: i32) -> i32;
    pub fn ioctl(fd: i32, req: u64, ...) -> i32;
    pub fn mmap(
        addr: *const u8,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *const u8;
    pub fn munmap(addr: *const u8, length: usize) -> i32;
}
