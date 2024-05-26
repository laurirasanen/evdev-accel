use clap::Parser;
use evdev::{
    raw_stream::RawDevice, uinput::VirtualDeviceBuilder, EventType, InputEvent, RelativeAxisType,
    Synchronization,
};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;
use std::io::prelude::*;
use std::{env, path::Path, time::SystemTime};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long)]
    device_name: Option<String>,
}

// TODO default values if no cfg file
#[derive(Deserialize, Debug)]
struct Config {
    sensitivity: f32,
    accel: f32,
    pre_scale: f32,
    post_scale: f32,
}

fn device_valid(device: &RawDevice) -> bool {
    let axes = device.supported_relative_axes();
    return axes.map_or(false, |axes| {
        axes.contains(RelativeAxisType::REL_X) && axes.contains(RelativeAxisType::REL_Y)
    });
}

fn pick_device(device_name: Option<String>) -> Option<RawDevice> {
    let devices = evdev::raw_stream::enumerate()
        .map(|t| t.1)
        .filter(|d| device_valid(d))
        .collect::<Vec<_>>();

    if devices.len() == 0 {
        println!("No valid devices found. Are you in the 'input' user group?");
        return None;
    }

    // Select by passed name
    if let Some(name) = device_name {
        let mut iter = devices.into_iter();
        let d = iter.find(|d| d.name().unwrap() == name);
        if d.is_some() {
            return d;
        }
        println!("Couldn't find a valid device named {}", name);
        for d in iter {
            println!("Valid devices:");
            println!("{}", d.name().unwrap_or("Unnamed device"));
        }
        return None;
    }

    // Select interactively
    for (i, d) in devices.iter().enumerate() {
        println!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
    }
    print!("Select the device [0-{}]: ", devices.len());
    let _ = std::io::stdout().flush();

    let mut chosen = String::new();
    std::io::stdin().read_line(&mut chosen).unwrap();
    let n = chosen.trim().parse::<usize>().unwrap();
    return Some(devices.into_iter().nth(n).unwrap());
}

fn main() {
    let args = CliArgs::parse();

    let home_path = env::var_os("HOME").expect("Failed to find HOME path");
    let cfg_path = Path::new(&home_path).join(".config/evdev-accel/config.toml");
    let config: Config = Figment::from(Toml::file(cfg_path))
        .extract()
        .expect("Failed to load config.toml");
    println!("{config:?}");

    let mut device = pick_device(args.device_name).expect("Failed to get device");
    println!("Device:");
    println!("{device:?}");

    device.grab().expect("Could not grab device");

    let mut virt_device = VirtualDeviceBuilder::new()
        .expect("Failed to create virtual device builder")
        .name("evdev-accel virtual device")
        .with_relative_axes(
            device
                .supported_relative_axes()
                .expect("Failed to get relative axis from device"),
        )
        .expect("Failed to add relative axis to virtual device")
        .with_keys(
            device
                .supported_keys()
                .expect("Failed to get supported keys from device"),
        )
        .expect("Failed to add supported keys to virtual device")
        .with_msc(
            device
                .misc_properties()
                .expect("Failed to get misc properties from device"),
        )
        .expect("Failed to add misc properties to virtual device")
        .build()
        .expect("Failed to build virtual device");

    let mut prev_time = SystemTime::now();
    let mut delta_time_ms: f32;
    let mut carry_x: f32 = 0.0;
    let mut carry_y: f32 = 0.0;
    let mut sync_x: i32 = 0;
    let mut sync_y: i32 = 0;
    let mut synced: bool = false;
    let mut sync_events: Vec<InputEvent> = Vec::new();
    loop {
        for ev in device.fetch_events().unwrap() {
            let mut drop = false;
            match ev.kind() {
                evdev::InputEventKind::RelAxis(axis) => match axis {
                    RelativeAxisType::REL_X => {
                        sync_x += ev.value();
                        drop = true;
                    }
                    RelativeAxisType::REL_Y => {
                        sync_y += ev.value();
                        drop = true;
                    }
                    _ => {}
                },
                evdev::InputEventKind::Synchronization(sync) => match sync {
                    Synchronization::SYN_REPORT => {
                        synced = true;
                        drop = true;
                    }
                    _ => {}
                },
                _ => {}
            }

            if !drop {
                sync_events.push(ev);
            }
        }

        if synced {
            // TODO do we care about the actual time inside events?
            let time = SystemTime::now();
            // leetmouse clamps lower limit to 1;
            // not sure if we need to when dealing with evdev instead of usb driver...?
            // lowest deltas between SYN_REPORTs seem pretty close to 1 on my 1000hz device.
            // This might feel wrong if gamer society ever invents 2000hz mice.
            delta_time_ms =
                (time.duration_since(prev_time).unwrap().as_secs_f32() * 1000.0).clamp(1.0, 100.0);
            prev_time = time;
            let (x, y) = accelerate(
                sync_x,
                sync_y,
                &mut carry_x,
                &mut carry_y,
                &config,
                delta_time_ms,
            );
            synced = false;
            sync_x = 0;
            sync_y = 0;

            if x != 0 {
                sync_events.push(InputEvent::new_now(
                    EventType::RELATIVE,
                    RelativeAxisType::REL_X.0,
                    x,
                ));
            }
            if y != 0 {
                sync_events.push(InputEvent::new_now(
                    EventType::RELATIVE,
                    RelativeAxisType::REL_Y.0,
                    y,
                ));
            }

            virt_device.emit(&sync_events).unwrap();
            sync_events.clear();
        }
    }
}

fn accelerate(
    x_og: i32,
    y_og: i32,
    carry_x: &mut f32,
    carry_y: &mut f32,
    config: &Config,
    delta_time_ms: f32,
) -> (i32, i32) {
    //println!("x {x_og} y {y_og} cx {carry_x} cy {carry_y} dt {delta_time_ms}");
    let mut accel_sens = config.sensitivity;
    let mut x_f = x_og as f32;
    let mut y_f = y_og as f32;

    x_f *= config.pre_scale;
    y_f *= config.pre_scale;

    let velocity = f32::sqrt(x_f * x_f + y_f * y_f);
    let rate = velocity / delta_time_ms;

    if rate > 0.0 {
        accel_sens += rate * config.accel;
    }

    accel_sens /= config.sensitivity;

    x_f *= accel_sens;
    y_f *= accel_sens;
    x_f *= config.post_scale;
    y_f *= config.post_scale;
    x_f += *carry_x;
    y_f += *carry_y;

    let x = x_f.floor() as i32;
    let y = y_f.floor() as i32;
    *carry_x = x_f - x as f32;
    *carry_y = y_f - y as f32;

    //println!("-> x {x} y {y} cx {carry_x} cy {carry_y}");
    return (x, y);
}
