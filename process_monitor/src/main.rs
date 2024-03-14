use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{self, Duration};

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

fn monitors_from_json(file_path: &str) -> Result<Monitors, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let monitors: Monitors = serde_json::from_str(&contents)
        .map_err(|e| format!("Error JSON: {}\nJSON content:\n{}", e, contents))?;

    Ok(monitors)
}

fn random_result() -> Option<ResultData> {
    let value = Some(rand::random::<i32>());
    let processed_at = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;

    Some(ResultData {
        value,
        processed_at,
    })
}

async fn update_monitors(monitors: Arc<Mutex<Monitors>>) {
    loop {
        {
            let mut monitors = monitors.lock().unwrap();
            for monitor in monitors.monitors.iter_mut() {
                monitor.result = random_result();
            }
        }

        time::sleep(Duration::from_secs(30)).await;
    }
}

async fn store_monitors(interval: u64, monitors: Arc<Mutex<Monitors>>) {
    loop {
        let current_time = SystemTime::now();
        let _timestamp = current_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let formatted_time = chrono::Local::now().format("%Y%m%d%H%M").to_string();
        let file_name = format!("{}_monitors.json", formatted_time);

        let json_string = {
            let monitors = monitors.lock().unwrap();
            serde_json::to_string_pretty(&*monitors).unwrap()
        };

        let output_file_path = format!(
            "D:/rust-assessment/process_monitor/storethemonitors/{}",
            file_name
        );
        let mut output_file = File::create(output_file_path.clone()).unwrap();
        output_file.write_all(json_string.as_bytes()).unwrap();

        println!("Stored at: {}", file_name);

        match File::open(&output_file_path) {
            Ok(mut file) => {
                let mut contents = String::new();
                if let Err(err) = file.read_to_string(&mut contents) {
                    eprintln!("Error  file {}: {}", &output_file_path, err);
                }
                match serde_json::from_str::<Monitors>(&contents) {
                    Ok(_) => println!("File {} formatted JSON", &output_file_path),
                    Err(err) => eprintln!("Error parsing file {}: {}", &output_file_path, err),
                }
            }
            Err(err) => eprintln!("Error opening file {}: {}", &output_file_path, err),
        }

        time::sleep(Duration::from_secs(interval)).await;
    }
}

async fn process_monitors(
    monitor_file_path: &str,
    _update_interval: u64,
    store_interval: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let monitors = Arc::new(Mutex::new(monitors_from_json(monitor_file_path)?));

    let update_task = update_monitors(Arc::clone(&monitors));
    let store_task = store_monitors(store_interval, Arc::clone(&monitors));

    tokio::join!(update_task, store_task);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("process_monitor")
        .arg(
            Arg::with_name("monitorFile")
                .short('m')
                .long("monitorFile")
                .value_name("FILE")
                .help("Sets the path to the monitors JSON file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let monitor_file_path = matches.value_of("monitorFile").unwrap();

    tokio::runtime::Runtime::new()?.block_on(async {
        let _ = tokio::time::timeout(Duration::from_secs(300), async {
            process_monitors(monitor_file_path, 30, 60).await
        })
        .await
        .unwrap_or_else(|e| {
            eprintln!("Completed five minutes : {}", e);
            std::process::exit(1);
        });
    });

    Ok(())
}
