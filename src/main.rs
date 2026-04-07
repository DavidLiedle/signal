use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct Signals {
    signals: BTreeMap<String, Vec<String>>,
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

fn default_signals() -> BTreeMap<String, Vec<String>> {
    BTreeMap::from([
        ("anxious".into(), vec![
            "Step outside for 5 minutes and breathe slowly.".into(),
            "Name three things you can see, hear, and feel right now.".into(),
        ]),
        ("bored".into(), vec![
            "Pick one small thing you've been putting off and do just the first step.".into(),
            "Learn something new for 15 minutes — a language, a tool, anything.".into(),
        ]),
        ("distracted".into(), vec![
            "Close all tabs, pick one task, set a 25-minute timer.".into(),
            "Put your phone in another room for the next hour.".into(),
        ]),
        ("lonely".into(), vec![
            "Text one person you haven't talked to in a while.".into(),
            "Go somewhere public — a cafe, a park — just to be around people.".into(),
        ]),
        ("overwhelmed".into(), vec![
            "Write down everything on your mind, then pick just one item.".into(),
            "Cancel or postpone one thing on today's list — give yourself room.".into(),
        ]),
        ("restless".into(), vec![
            "Go for a short walk — even 10 minutes resets your head.".into(),
            "Do something physical for 5 minutes: stretch, push-ups, jump rope.".into(),
        ]),
        ("sad".into(), vec![
            "Put on a song you love and let yourself feel it.".into(),
            "Write down one good thing that happened recently, no matter how small.".into(),
        ]),
        ("stuck".into(), vec![
            "Describe the problem out loud as if explaining it to a friend.".into(),
            "Work on a different part of the problem and come back to this later.".into(),
        ]),
        ("tired".into(), vec![
            "Take a 20-minute nap or splash cold water on your face.".into(),
            "Drink a full glass of water — dehydration feels a lot like fatigue.".into(),
        ]),
    ])
}

fn simple_random(bound: usize) -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize;
    let pid = process::id() as usize;
    (nanos ^ pid) % bound
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

fn random_suggestion(s: &Signals, mood_filter: Option<&str>) {
    let suggestions: Vec<&str> = if let Some(mood) = mood_filter {
        let key = mood.to_lowercase();
        match s.signals.get(&key) {
            Some(v) => v.iter().map(|s| s.as_str()).collect(),
            None => {
                eprintln!("Unknown mood: {key}");
                eprintln!("Run `signal list` to see available moods.");
                process::exit(1);
            }
        }
    } else {
        s.signals.values().flat_map(|v| v.iter()).map(|s| s.as_str()).collect()
    };

    if suggestions.is_empty() {
        eprintln!("No signals defined. Use `signal add <mood> <message>` to add one.");
        process::exit(1);
    }

    let idx = simple_random(suggestions.len());
    println!("{}", suggestions[idx]);
}

fn print_usage() {
    eprintln!("signal — mood-to-action mapper\n");
    eprintln!("Usage:");
    eprintln!("  signal                       Random suggestion from all moods");
    eprintln!("  signal --mood <mood>         Random suggestion for a specific mood");
    eprintln!("  signal <mood>                Show all suggestions for a mood");
    eprintln!("  signal add <mood> <msg>      Add a suggestion to a mood");
    eprintln!("  signal remove <mood> [msg]   Remove a mood or a specific suggestion");
    eprintln!("  signal list                  List all moods and suggestions");
    eprintln!("  signal path                  Print the data file path");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        let s = load();
        random_suggestion(&s, None);
        return;
    }

    // Check for --mood flag
    if args[0] == "--mood" {
        if args.len() < 2 {
            eprintln!("Usage: signal --mood <mood>");
            process::exit(1);
        }
        let s = load();
        random_suggestion(&s, Some(&args[1]));
        return;
    }

    let cmd = args[0].to_lowercase();

    match cmd.as_str() {
        "list" => {
            let s = load();
            if s.signals.is_empty() {
                println!("No signals defined. Use `signal add <mood> <message>` to add one.");
            } else {
                for (mood, suggestions) in &s.signals {
                    println!("  {mood}:");
                    for sg in suggestions {
                        println!("    - {sg}");
                    }
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
            s.signals.entry(mood.clone()).or_default().push(msg);
            save(&s);
            println!("Added to: {mood}");
        }
        "remove" => {
            if args.len() < 2 {
                eprintln!("Usage: signal remove <mood> [message]");
                process::exit(1);
            }
            let mood = args[1].to_lowercase();
            let mut s = load();
            if args.len() == 2 {
                if s.signals.remove(&mood).is_some() {
                    save(&s);
                    println!("Removed mood: {mood}");
                } else {
                    eprintln!("Not found: {mood}");
                    process::exit(1);
                }
            } else {
                let msg = args[2..].join(" ");
                if let Some(suggestions) = s.signals.get_mut(&mood) {
                    let before = suggestions.len();
                    suggestions.retain(|s| s != &msg);
                    if suggestions.len() < before {
                        if suggestions.is_empty() {
                            s.signals.remove(&mood);
                        }
                        save(&s);
                        println!("Removed suggestion from: {mood}");
                    } else {
                        eprintln!("Suggestion not found in: {mood}");
                        process::exit(1);
                    }
                } else {
                    eprintln!("Not found: {mood}");
                    process::exit(1);
                }
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
            if let Some(suggestions) = s.signals.get(&key) {
                for sg in suggestions {
                    println!("{sg}");
                }
            } else {
                eprintln!("Unknown mood: {key}");
                eprintln!("Run `signal list` to see available moods.");
                process::exit(1);
            }
        }
    }
}
