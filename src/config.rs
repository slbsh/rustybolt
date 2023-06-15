use std::{env, fs};
use std::error::Error;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use toml;

lazy_static! {
    static ref CONFIG: Mutex<Option<Conf>> = Mutex::new(None);
}
// struct to load the config into
#[derive(Deserialize, Serialize, Clone)]
pub struct Conf {
    pub roll_channel: u64,
    pub min_points: u16,
    pub max_points: u16,
    pub teams: u8,
    pub players: Vec<String>,
    pub countries: Vec<String>,
}

// reload config func
pub async fn init_config() -> Result<(), Box<dyn Error>> {
    let mut config = CONFIG.lock().await;

    // check env var, if empty pick the default
    let config_file = env::var("POT_CONFIG")
        .unwrap_or("config.toml".to_string());

    // load from a file
    let contents = fs::read_to_string(config_file)?;
        
    // return the parsed struct
    let parsed_config = toml::from_str::<Conf>(&contents)?.clone();

    // modify the config
    *config = Some(parsed_config);

    Ok(())
}

pub async fn get_config() -> Result<Conf, Box<dyn Error>> {
    let config = CONFIG.lock().await;
    // try retrieving config
    if let Some(config) = &*config {
        Ok(config.clone())
    } else {
        // if all fails...
        Err("Err: Config not Initialized".into())
    }
}

pub async fn modify_config(new_conf: Conf) -> Result<(), Box<dyn Error>> {
    // read config and modify it to the new state
    let mut conf = CONFIG.lock().await;
    *conf = Some(new_conf);

    // parse into toml
    let parsed_new = toml::to_string(&*conf)?;

    // write the parsed toml to a file
    fs::write("config.toml", parsed_new)?;

    Ok(())
}
