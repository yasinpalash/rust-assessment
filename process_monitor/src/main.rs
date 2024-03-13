use serde::{Serialize, Deserialize};
use serde_json;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct ResultData {
    value: Option<i32>, 
    processed_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Monitor {
    name: String,
    monitor_id: Option<u32>,
    script: Option<String>, 
    result: Option<ResultData>,
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Monitors {
    monitors: Vec<Monitor>,
}

fn read_monitors_from_json(file_path: &str) -> Result<Monitors, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let monitors: Monitors = serde_json::from_str(&contents)
        .map_err(|e| format!("Error JSON: {}\nJSON content:\n{}", e, contents))?;

    Ok(monitors)
}

fn generate_random_result() -> Result<Option<ResultData>, Box<dyn std::error::Error>> {
    let value = Some(rand::random::<i32>());
    let processed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Error getting : {}", e))?
        .as_secs() as i64;

    Ok(Some(ResultData { value, processed_at }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path/to/monitors.json>", args[0]);
        std::process::exit(1);
    }

    let monitor_file_path = &args[1];

    let mut monitors = read_monitors_from_json(monitor_file_path)?;

    for monitor in monitors.monitors.iter_mut() {
        monitor.result = generate_random_result()?;
    }

    let json_string = serde_json::to_string_pretty(&monitors)?;

    println!("Updated JSON:\n{}", json_string);

    let output_file_path = "D:/rust-assessment/process_monitor/forwrite/back.json"; 
    let mut output_file = File::create(output_file_path)?;
    output_file.write_all(json_string.as_bytes())?;

    println!("Write Json : {}", output_file_path);

    Ok(())
}
