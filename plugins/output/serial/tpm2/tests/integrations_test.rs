use lightoros_output_serial_tpm2;
use lightoros_plugin::{RgbData, RGB};
use std::time::{Duration};
use std::thread;
use serde_json::json;

const port: &str = "/dev/tty.usbmodem14101";

#[test]
fn test_get_info() {
    let plugin_info = lightoros_output_serial_tpm2::info();
    assert_eq!(plugin_info.name, "SerialOutputTPM2");
    assert_eq!(plugin_info.api_version, 1);
    assert_eq!(plugin_info.filename, "lightoros_output_serial_tpm2");
}

#[test]
fn test_create() {
    let config = json!({
        "port": ""
    });
    let _plugin = lightoros_output_serial_tpm2::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_empty_config() {
    let config = json!({
    });
    lightoros_output_serial_tpm2::create(&config);
}

#[test]
#[should_panic]
fn test_create_with_wrong_dattype_in_config() {
    let config = json!({
        "test": true
    });
    lightoros_output_serial_tpm2::create(&config);
}

#[test]
fn test_send_to_invalid_port() {
    let config = json!({
        "port": "invalid"
    });
    let plugin = lightoros_output_serial_tpm2::create(&config);
    let out: RgbData = Vec::with_capacity(1);
    let result = plugin.send(&out);
    assert!(!result);
}

// /dev/tty.usbmodem14201 

#[test]
fn test_send_empty_data() {
    let config = json!({
        "port": port
    });
    let plugin = lightoros_output_serial_tpm2::create(&config);
    let out: RgbData = Vec::new();
    thread::sleep(Duration::from_millis(1000));
    let result = plugin.send(&out);
    assert!(result);
}

#[test]
fn test_send_data_red_dot() {
    let config = json!({
        "port": port
    });
    let plugin = lightoros_output_serial_tpm2::create(&config);
    let mut out: RgbData = Vec::with_capacity(1);
    out.push(RGB {
                r: 255,
                g: 0,
                b: 0
            });
    thread::sleep(Duration::from_millis(1000));
    let result = plugin.send(&out);
    assert!(result);
}

#[test]
fn test_send_data_blue_dot() {
    let config = json!({
        "port": port
    });
    let plugin = lightoros_output_serial_tpm2::create(&config);
    let mut out: RgbData = Vec::with_capacity(1);
    out.push(RGB {
                r: 0,
                g: 0,
                b: 255
            });
    thread::sleep(Duration::from_millis(1000));       
    let result = plugin.send(&out);
    assert!(result);
}

#[test]
fn test_send_data_green_dot() {
    let config = json!({
        "port": port
    });
    let plugin = lightoros_output_serial_tpm2::create(&config);
    let mut out: RgbData = Vec::with_capacity(1);
    out.push(RGB {
                r: 0,
                g: 255,
                b: 0
            });
    thread::sleep(Duration::from_millis(1000));            
    let result = plugin.send(&out);
    assert!(result);
}