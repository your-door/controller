//! Discover Bluetooth devices and list them.

mod config;
mod trigger;
mod error;
mod database;

use aes::{Aes128, cipher::{KeyInit, generic_array::GenericArray, BlockDecrypt, typenum}};
use bluer::{Adapter, AdapterEvent, Address};
use byteorder::ByteOrder;
use chrono::{Datelike, Weekday};
use clap::Parser;
use futures::{pin_mut, StreamExt};
use hex::FromHex;
use core::time;
use std::fs;
use log::{debug, error, info, warn};

async fn get_from_hex_array(str: &str) -> error::Result<Vec<u8>> {
    let splits = str.split(", ");
    let mut arr: Vec<u8> = Vec::new();

    for s in splits {
        let hex_str = &s[2..4];
        let a = Vec::from_hex(hex_str)
            .or(Err(error::new("could not convert from hex".to_string())))?;
        let val = a.get(0)
            .ok_or(error::new("hex array has no 0 index".to_string()))?;
        arr.push(*val);
    }

    Ok(arr)
}

async fn get_time(str: &str) -> error::Result<chrono::NaiveTime> {
    let mut splits = str.split(":");

    let hours = splits.next()
        .unwrap()
        .parse::<u32>()
        .or(Err(error::new("could not parse hours to int".to_string())))?;

    let minutes = splits.next()
        .unwrap()
        .parse::<u32>()
        .or(Err(error::new("could not parse minutes to int".to_string())))?;

    let seconds = splits.next()
        .unwrap()
        .parse::<u32>()
        .or(Err(error::new("could not parse seconds to int".to_string())))?;

    let time_obj = chrono::NaiveTime::from_hms(hours, minutes, seconds);
    Ok(time_obj)
}

async fn query_device(adapter: &Adapter, addr: Address, config: &mut config::Config) -> error::Result<()> {
    let device = adapter.device(addr)
        .or(Err(error::new(format!("could not find device from addr: {}", addr))))?;

    let formated_addr = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]);

    // Check if we have MD
    let md_res = device.manufacturer_data().await;
    if let Err(_err) = md_res {
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone(), formated_addr.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    let md_opt = md_res.unwrap();
    if let None = md_opt {
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone(), formated_addr.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    let md = md_opt.unwrap();

    if md.contains_key(&89) {
        info!("Address: {}, Address type: {}. Name: {:?}, RSSI {:?}, Connected: {:?}, Paired: {:?}, Services: {:?}, MD: {:?}", addr, device.address_type().await?, 
        device.name().await?, device.rssi().await?, device.is_connected().await?, device.is_paired().await?, device.uuids().await?, device.manufacturer_data().await?);
    }

    // Check if we have a config for that device
    let c = config.devices.get_mut(&formated_addr.clone());
    if let None = c {
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone().clone(), formated_addr.clone().clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    let device_config = c.unwrap();

    // Check if we can read the key from config
    let decoded_key_res = get_from_hex_array(&device_config.key).await;
    if let Err(err) = decoded_key_res {
        warn!("{} has a device key configured which can't be decoded: {}", formated_addr.clone(), err);
        
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    let device_key = decoded_key_res.unwrap();

    // Check if key is correct length
    if device_key.len() != 16 {
        warn!("{} has a device key configured which is not 16 bytes long", formated_addr.clone());
        
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    // Check if we have the correct manufacture data
    let md_sel = md.get(&device_config.manufacture);
    if let None = md_sel {
        warn!("{} presented wrong manufacture data key", formated_addr.clone());
        
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    let md_data = md_sel.unwrap();

    // Check if md_data is correct length
    if md_data.len() != 24 {
        warn!("{} presented invalid manufacture data length: {}", formated_addr.clone(), md_data.len());
        
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    // Init correct parameters for AES
    let key: GenericArray<u8, typenum::U16> = GenericArray::clone_from_slice(&device_key[..16]);
    let mut iv = [0x24; 16];
 
    // Copy over IV
    iv[..8].copy_from_slice(&md_data[..8]);
    iv[8..16].copy_from_slice(&md_data[..8]);
    
    // Copy over encrypted data
    let mut sl = [0; 16];
    sl.copy_from_slice(&md_data[8..24]);
    let mut buf = GenericArray::from(sl);
    
    // Create AES context and decrypt block
    let a = Aes128::new(&key);
    a.decrypt_block(&mut buf);
    
    // XOR with IV
    for n in 0..16 {
        buf[n] = buf[n] ^ iv[n];
    }

    // Check for device id
    let device_id = get_from_hex_array(&device_config.device_id).await?;
    for i in 0..10 {
        let di = device_id.get(i)
            .ok_or(error::new("device_id has no index".to_string()))?;
        if buf[i] != *di {
            warn!("{} presented invalid device ID", formated_addr.clone());
            trigger::trigger_off(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
            return Ok(());
        }
    }

    // Check for restart counter
    // TODO


    // Check for time
    let start = chrono::Utc::now();
    let since_the_epoch = start.timestamp() as u64;
    let time = byteorder::BE::read_u32(&buf[12..16]);

    // Get time from database
    let timedto = database::get_times(config.database_path.clone(), formated_addr.clone().clone()).await?;

    // Check for time diff
    let diff = since_the_epoch - timedto.last_seen_local;
    let diff_tag = time - timedto.last_seen;

    // Update internal config
    if time > timedto.last_seen {
        debug!("{}: Local time diff: {}, Tag time diff: {}", formated_addr.clone(), diff, diff_tag);

        database::store_times(config.database_path.clone(), formated_addr.clone().clone(), since_the_epoch, time).await?;

        debug!("{}: Storing decrypted time: {:?}", formated_addr.clone(), time);
    }

    let skew = diff.abs_diff(diff_tag as u64);
    if skew > config.allowed_skew as u64 {
        warn!("{} time was out of sync by {}", formated_addr.clone(), skew);
        
        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone().clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }            

    // We need to get the day
    let current_time = chrono::Local::now().naive_local();
    let day = match current_time.weekday() {
        Weekday::Mon => "monday",
        Weekday::Tue => "tuesday",
        Weekday::Wed => "wednesday",
        Weekday::Thu => "thursday",
        Weekday::Fri => "friday",
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
    };

    // Check if we have a config for the day
    let times_option = device_config.allowed_times.get(day);
    if let None = times_option {
        info!("{} wanted to get access on a non configured day", formated_addr.clone());

        // Ensure that devices with no config are not triggered
        trigger::trigger_off(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
        return Ok(());
    }

    let times = times_option.unwrap();
    let current_time_local = current_time.time();

    for time in times {
        let mut splits = time.split("-");

        let start = splits.next();
        let end = splits.next();

        if let Some(start_str) = start {
            if let Some(end_str) = end {
                let start_time = get_time(start_str).await?;
                let end_time = get_time(end_str).await?;

                if current_time_local >= start_time && current_time_local <= end_time {
                    info!("{} is allowed. Triggering", formated_addr.clone());
                    trigger::trigger_on(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
                    return Ok(());
                }
            }
        }
    }

    info!("{} has no access this time of the day", formated_addr.clone());
    trigger::trigger_off(formated_addr.clone(), device_config.name.clone(), config.home_assistant.clone()).await?;
    Ok(())
}

async fn start_ble(config: &mut config::Config) -> error::Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.adapter("hci0")?;
    adapter.set_powered(true).await?;

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    let device_events = adapter.discover_devices_with_changes().await?;
    pin_mut!(device_events);

    loop {
        match device_events.next().await {
            Some(device_event) => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        let res = query_device(&adapter, addr, config).await;
                        if let Err(err) = res {
                            error!("Error in discovery with {}: {}", addr, &err);

                            let formated_addr = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]);
                            trigger::trigger_off(formated_addr.clone(), formated_addr.clone(), config.home_assistant.clone().clone()).await?;
                        }
                    }
                    AdapterEvent::DeviceRemoved(addr) => {
                        debug!("Device removed: {}", addr);

                        let formated_addr = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]);
                        trigger::trigger_off(formated_addr.clone(), formated_addr.clone(), config.home_assistant.clone().clone()).await?;
                    }
                    _ => (),
                }
            }
            _ => ()
        }
    }
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration path
    #[clap(short, long)]
    config: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> error::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

    let args = Args::parse();

    // Load configuration from disk
    let file_content = fs::read_to_string(args.config)?;
    let config_result: serde_yaml::Result<config::Config> = serde_yaml::from_str(&file_content);
    if config_result.is_ok() {
        let mut config = config_result.unwrap();
        info!("Configuration: {}", config);

        // Start database
        database::init_database(&mut config).await?;
        start_ble(&mut config).await?;
    } else {
        let e = config_result.err().unwrap();
        error!("Could not read config: {}", &e);
    }
    
    loop{
        tokio::time::sleep(time::Duration::from_secs(60)).await;
    }
}