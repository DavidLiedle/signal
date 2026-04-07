use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process;

#[derive(Serialize, Deserialize)]
struct Signals {
    signals: BTreeMap<String, String>,
}

fn data_path() -> PathBuf {
    if let Ok(p) = std::env::var("SIGNAL_DATA") {
        return PathBuf::from(p);
    }
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("signal");
    fs::create_dir_all(&dir).ok();
    dir.push("signals.json");
    dir
}

fn default_signals() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("anxious".into(), "Step outside for 5 minutes and breathe slowly.".into()),
        ("bored".into(), "Pick one small thing you've been putting off and do just the first step.".into()),
        ("distracted".into(), "Close all tabs, pick one task, set a 25-minute timer.".into()),
        ("lonely".into(), "Text one person you haven't talked to in a while.".into()),
        ("overwhelmed".into(), "Write down everything on your mind, then pick just one item.".into()),
        ("restless".into(), "Go for a short walk — even 10 minutes resets your head.".into()),
        ("sad".into(), "Put on a song you love and let yourself feel it.".into()),
        ("stuck".into(), "Describe the problem out loud as if explaining it to a friend.".into()),
        ("tired".into(), "Take a 20-minute nap or splash cold water on your face.".into()),
    ])
}

fn load() -> Signals {
    let path = data_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("error: failed to read {}: {e}", path.display());
            process::exit(1);
        });
        serde_json::from_str(&data).unwrap_or_else(|e| {
            eprintln!("error: failed to parse {}: {e}", path.display());
            process::exit(1);
        })
    } else {
        let s = Signals { signals: default_signals() };
        save(&s);
        s
    }
}

fn save(signals: &Signals) {
    let path = data_path();
    let json = serde_json::to_string_pretty(signals).expect("failed to serialize");
    fs::write(&path, json).unwrap_or_else(|e| {
        eprintln!("error: failed to write {}: {e}", path.display());
        process::exit(1);
    });
}

fn print_usage() {
    eprintln!("signal — mood-to-action mapper\n");
    eprintln!("Usage:");
    eprintln!("  signal <mood>           Look up a suggestion for a mood");
    eprintln!("  signal add <mood> <msg>  Add or update a mood entry");
    eprintln!("  signal remove <mood>     Remove a mood entry");
    eprintln!("  signal list              List all known moods");
    eprintln!("  signal path              Print the data file path");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        print_usage();
        process::exit(1);
    }

    let cmd = args[0].to_lowercase();

    match cmd.as_str() {
        "list" => {
            let s = load();
            if s.signals.is_empty() {
                println!("No signals defined. Use `signal add <mood> <message>` to add one.");
            } else {
                for (mood, suggestion) in &s.signals {
                    println!("  {mood:<15} {suggestion}");
                }
            }
        }
        "add" => {
            if args.len() < 3 {
                eprintln!("Usage: signal add <mood> <message>");
                process::exit(1);
            }
            let mood = args[1].to_lowercase();
            let msg = args[2..].join(" ");
            let mut s = load();
            s.signals.insert(mood.clone(), msg);
            save(&s);
            println!("Added: {mood}");
        }
        "remove" => {
            if args.len() < 2 {
                eprintln!("Usage: signal remove <mood>");
                process::exit(1);
            }
            let mood = args[1].to_lowercase();
            let mut s = load();
            if s.signals.remove(&mood).is_some() {
                save(&s);
                println!("Removed: {mood}");
            } else {
                eprintln!("Not found: {mood}");
                process::exit(1);
            }
        }
        "path" => {
            println!("{}", data_path().display());
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        mood => {
            let s = load();
            let key = mood.to_lowercase();
            if let Some(suggestion) = s.signals.get(&key) {
                println!("{suggestion}");
            } else {
                eprintln!("Unknown mood: {key}");
                eprintln!("Run `signal list` to see available moods.");
                process::exit(1);
            }
        }
    }
}
