#![cfg(target_os = "android")]
#![allow(non_snake_case)]

use jni::objects::{JObject, JString};
use jni::JNIEnv;
use std::sync::MutexGuard;

use lightoros_engine::*;

use log::*;

#[no_mangle]
pub unsafe extern "system" fn Java_de_gamadril_lightoros_MainActivity_createEngine(
    env: JNIEnv,
    obj: JObject,
) {
    //android_logger::init_once(Config::default().with_min_level(Level::Debug));

    debug!("----------- CREATE ------------");
    let engine = LightorosEngine::new();
    env.set_rust_field(obj, "enginePtr", engine).unwrap();
}

pub unsafe extern "system" fn Java_de_gamadril_lightoros_MainActivity_destroyEngine(
    env: JNIEnv,
    obj: JObject,
) {
    debug!("----------- DESTROY ------------");
    let _: jni::errors::Result<LightorosEngine> = env.take_rust_field(obj, "enginePtr");
}

#[no_mangle]
pub unsafe extern "system" fn Java_de_gamadril_lightoros_MainActivity_initEngine(
    env: JNIEnv,
    obj: JObject,
    config: JString,
    plugins_dir: JString,
) {
    let config: String = env
        .get_string(config)
        .expect("Couldn't get config string")
        .into();
    let plugins_dir: String = env
        .get_string(plugins_dir)
        .expect("Couldn't get plugins dir")
        .into();

    let mut engine: MutexGuard<LightorosEngine> = env.get_rust_field(obj, "enginePtr").unwrap();

    debug!("----------- INIT ------------");

    //debug!("----------- CONFIG: {} ------------", config);
    //debug!("----------- PLUGINS_DIR: {} ------------", plugins_dir);

    
    let result = engine.init(config, plugins_dir);

    if let Err(e) = result {
        error!("ERROR: {}", e);
    } else {
        debug!("INIT DONE!");
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_de_gamadril_lightoros_MainActivity_startEngine(
    env: JNIEnv,
    obj: JObject,
) {
    debug!("----------- START ------------");
    let mut engine: MutexGuard<LightorosEngine> = env.get_rust_field(obj, "enginePtr").unwrap();
    engine.start();
}
