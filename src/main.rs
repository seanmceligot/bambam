mod config;
mod wait;

use anyhow::{Context, Result};

use clap::{App, Arg};
use config::read_bambam_config;
use config::BamBamConfig;
use porcupine::PorcupineBuilder;
use pv_recorder::PvRecorder;
use pv_recorder::PvRecorderBuilder;
use rhino::Rhino;
use rhino::RhinoBuilder;
use std::env;
use std::path::Path;
use std::path::PathBuf;

use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use wait::Wait;

static LISTENING: AtomicBool = AtomicBool::new(false);

fn expand_config_path(config_path: &String) -> Result<PathBuf> {
    let expanded_path = shellexpand::full(config_path)
        .with_context(|| format!("Error expanding path: {:?}", config_path))?;

    Ok(PathBuf::from(expanded_path.to_string()))
}
fn bambam_listen(dev: i32) -> Result<()> {
    let config_path = home_path(".config/bambam/config.json")?;
    let config = read_bambam_config(&config_path)
        .with_context(|| format!("could not read {:?}", config_path))?;

    //let keyword_paths = home_path(".config/bambam/bam-bam_en_linux_v3_0_0.ppn")?;
    let keyword_paths: PathBuf = expand_config_path(&config.ppn_file)?;

    let keywords_or_paths = [PathBuf::from(keyword_paths)];
    let porcupine_builder =
        PorcupineBuilder::new_with_keyword_paths(config.access_key.clone(), &keywords_or_paths);

    let context_path = expand_config_path(&config.rhn_file)?;
    let rhino_builder = RhinoBuilder::new(config.access_key.clone(), context_path);
    let rhino = rhino_builder.init().expect("Failed to create Rhino");

    let porcupine = porcupine_builder
        .init()
        .expect("Failed to create Porcupine");

    let recorder = PvRecorderBuilder::new(porcupine.frame_length() as i32)
        .device_index(dev)
        .init()
        .expect("Failed to initialize pvrecorder");
    recorder.start().expect("Failed to start audio recording");

    LISTENING.store(true, Ordering::SeqCst);
    ctrlc::set_handler(|| {
        LISTENING.store(false, Ordering::SeqCst);
    })
    .expect("Unable to setup signal handler");

    #[rustfmt::skip]  // works in stable
    let mut wait = Wait::new(vec![
        "●    ",
        " ●    ",
        "  ●   ",
        "   ●  ",
        "    ● ",
        "     ●",
        "    ● ",
        "   ●  ",
        "  ●   ",
        " ●    ",
        "●     ",
    ]);

    while LISTENING.load(Ordering::SeqCst) {
        print!("\rsleeping {}", wait.next());
        let frame = recorder.read().expect("Failed to read audio frame");

        let keyword_index = porcupine.process(&frame).unwrap();
        if keyword_index >= 0 {
            println!("Detected {}", keyword_index);
            let mb_command = listen_for_command(&rhino, &recorder)?;
            if let Some(command) = mb_command {
                println!("command {}", command);
                process(&config, &command)?;
            }
        }
    }

    println!("\nStopping...");
    recorder.stop().expect("Failed to stop audio recording");
    Ok(())
}
fn run(script: &PathBuf) -> Result<()> {
    let mut cmd = Command::new(script);
    cmd.arg(script);
    println!("run {:?}", script);
    let rtn = cmd.status()?;
    println!("ran {:?}", script);
    println!("ran {:?} {}", script, rtn);
    Ok(())
}
fn process(config: &BamBamConfig, command: &String) -> Result<()> {
    match command.as_str() {
        "lock_door" => run(&expand_config_path(&config.lock_door)?),
        "kitchen_light_yellow" => run(&expand_config_path(&config.kitchen_light_yellow)?),
        "kitchen_light_purple" => run(&expand_config_path(&config.kitchen_light_purple)?),
        _ => {
            println!("unknown command {}", command);
            Ok(())
        }
    }
}
fn show_audio_devices() {
    let audio_devices = PvRecorderBuilder::default().get_available_devices();
    match audio_devices {
        Ok(audio_devices) => {
            for (idx, device) in audio_devices.iter().enumerate() {
                println!("index: {idx}, device name: {device:?}");
            }
        }
        Err(err) => panic!("Failed to get audio devices: {}", err),
    };
}

fn main() {
    let matches = App::new("Bam-Bam voice command")
        .arg(
            Arg::with_name("dev")
                .long("dev")
                .value_name("INDEX")
                .help("Index of input audio device.")
                .takes_value(true)
                .default_value("-1"),
        )
        .arg(Arg::with_name("show_audio_devices").long("show_audio_devices"))
        .get_matches();

    if matches.is_present("show_audio_devices") {
        return show_audio_devices();
    }

    let dev = matches.value_of("dev").unwrap().parse().unwrap();

    match bambam_listen(dev) {
        Ok(_) => println!("done"),
        Err(e) => println!("err {:?}", e),
    }
}

fn home_path(path: &str) -> Result<PathBuf> {
    let home = env::var("HOME").with_context(|| "HOME environment variable not set")?;
    let mut home_path = PathBuf::from(home); // Create a PathBuf from HOME
    home_path.push(path); // Append the provided path
    Ok(home_path) // Return the combined path
}

fn listen_for_command(rhino: &Rhino, recorder: &PvRecorder) -> Result<Option<String>> {
    println!("Listening for commands...");

    #[rustfmt::skip]  // works in stable
    let mut wait = Wait::new(vec![
        "👂    ",
        " 👂    ",
        "  👂   ",
        "   👂  ",
        "    👂 ",
        "     👂",
        "    👂 ",
        "   👂  ",
        "  👂   ",
        " 👂    ",
        "👂     ",
    ]);

    while LISTENING.load(Ordering::SeqCst) {
        //println!("record...");
        let frame = recorder.read().expect("Failed to read audio frame");
        print!("\rlisten for command {}", wait.next());

        let is_finalized = rhino.process(&frame).unwrap();
        if is_finalized {
            println!("finalized");
            let inference = rhino.get_inference()?;
            let mb_intent = inference.intent;
            if let Some(intent) = mb_intent {
                if inference.is_understood {
                    println!("intent : '{}'", intent);
                    for (slot, value) in inference.slots.iter() {
                        println!("slot: {}", slot);
                        println!("value: {}", value);
                    }
                    return Ok(Some(intent));
                } else {
                    println!("Did not understand the command");
                }
            }
        }
        //println!("command loop");
    }
    Ok(None)
}
