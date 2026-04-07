use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    text: String,
    #[serde(default)]
    last_used: u64,
}

#[derive(Serialize, Deserialize)]
struct Signals {
    signals: BTreeMap<String, Vec<Entry>>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
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

fn entry(text: &str) -> Entry {
    Entry { text: text.into(), last_used: 0 }
}

fn default_signals() -> BTreeMap<String, Vec<Entry>> {
    BTreeMap::from([
        ("anxious".into(), vec![
            entry("Step outside for 5 minutes and breathe slowly."),
            entry("Name three things you can see, hear, and feel right now."),
        ]),
        ("bored".into(), vec![
            entry("Pick one small thing you've been putting off and do just the first step."),
            entry("Learn something new for 15 minutes — a language, a tool, anything."),
        ]),
        ("distracted".into(), vec![
            entry("Close all tabs, pick one task, set a 25-minute timer."),
            entry("Put your phone in another room for the next hour."),
        ]),
        ("lonely".into(), vec![
            entry("Text one person you haven't talked to in a while."),
            entry("Go somewhere public — a cafe, a park — just to be around people."),
        ]),
        ("overwhelmed".into(), vec![
            entry("Write down everything on your mind, then pick just one item."),
            entry("Cancel or postpone one thing on today's list — give yourself room."),
        ]),
        ("restless".into(), vec![
            entry("Go for a short walk — even 10 minutes resets your head."),
            entry("Do something physical for 5 minutes: stretch, push-ups, jump rope."),
        ]),
        ("sad".into(), vec![
            entry("Put on a song you love and let yourself feel it."),
            entry("Write down one good thing that happened recently, no matter how small."),
        ]),
        ("stuck".into(), vec![
            entry("Describe the problem out loud as if explaining it to a friend."),
            entry("Work on a different part of the problem and come back to this later."),
        ]),
        ("tired".into(), vec![
            entry("Take a 20-minute nap or splash cold water on your face."),
            entry("Drink a full glass of water — dehydration feels a lot like fatigue."),
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

/// Pick a random index weighted toward least-recently-used entries.
/// Weight = (now - last_used + 1), so never-used items (last_used=0) get the
/// highest weight. The +1 avoids zero-weight for something just used.
fn weighted_random_index(entries: &[&Entry]) -> usize {
    let now = now_secs();
    let weights: Vec<u64> = entries.iter().map(|e| now - e.last_used + 1).collect();
    let total: u64 = weights.iter().sum();
    let mut pick = (simple_random(total as usize)) as u64;
    for (i, &w) in weights.iter().enumerate() {
        if pick < w {
            return i;
        }
        pick -= w;
    }
    entries.len() - 1
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

/// Collect matching entries as (mood_key, entry_index) pairs so we can update
/// the chosen entry's last_used timestamp in the original data structure.
fn random_suggestion(s: &mut Signals, mood_filter: Option<&str>) {
    let candidates: Vec<(String, usize)> = if let Some(mood) = mood_filter {
        let key = mood.to_lowercase();
        match s.signals.get(&key) {
            Some(v) => (0..v.len()).map(|i| (key.clone(), i)).collect(),
            None => {
                eprintln!("Unknown mood: {key}");
                eprintln!("Run `signal list` to see available moods.");
                process::exit(1);
            }
        }
    } else {
        s.signals
            .iter()
            .flat_map(|(k, v)| (0..v.len()).map(move |i| (k.clone(), i)))
            .collect()
    };

    if candidates.is_empty() {
        eprintln!("No signals defined. Use `signal add <mood> <message>` to add one.");
        process::exit(1);
    }

    let entry_refs: Vec<&Entry> = candidates
        .iter()
        .map(|(k, i)| &s.signals[k][*i])
        .collect();

    let pick = weighted_random_index(&entry_refs);
    let (ref mood_key, entry_idx) = candidates[pick];

    let chosen = &mut s.signals.get_mut(mood_key).unwrap()[entry_idx];
    println!("{}", chosen.text);
    chosen.last_used = now_secs();
    save(s);
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
        let mut s = load();
        random_suggestion(&mut s, None);
        return;
    }

    // Check for --mood flag
    if args[0] == "--mood" {
        if args.len() < 2 {
            eprintln!("Usage: signal --mood <mood>");
            process::exit(1);
        }
        let mut s = load();
        random_suggestion(&mut s, Some(&args[1]));
        return;
    }

    let cmd = args[0].to_lowercase();

    match cmd.as_str() {
        "list" => {
            let s = load();
            if s.signals.is_empty() {
                println!("No signals defined. Use `signal add <mood> <message>` to add one.");
            } else {
                for (mood, entries) in &s.signals {
                    println!("  {mood}:");
                    for e in entries {
                        if e.last_used > 0 {
                            println!("    - {} (last used: {})", e.text, format_timestamp(e.last_used));
                        } else {
                            println!("    - {}", e.text);
                        }
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
            s.signals.entry(mood.clone()).or_default().push(entry(&msg));
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
                if let Some(entries) = s.signals.get_mut(&mood) {
                    let before = entries.len();
                    entries.retain(|e| e.text != msg);
                    if entries.len() < before {
                        if entries.is_empty() {
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
            if let Some(entries) = s.signals.get(&key) {
                for e in entries {
                    println!("{}", e.text);
                }
            } else {
                eprintln!("Unknown mood: {key}");
                eprintln!("Run `signal list` to see available moods.");
                process::exit(1);
            }
        }
    }
}

fn format_timestamp(epoch: u64) -> String {
    // Simple human-readable relative time, no external deps
    let now = now_secs();
    if epoch > now {
        return "just now".into();
    }
    let delta = now - epoch;
    if delta < 60 {
        format!("{delta}s ago")
    } else if delta < 3600 {
        format!("{}m ago", delta / 60)
    } else if delta < 86400 {
        format!("{}h ago", delta / 3600)
    } else {
        format!("{}d ago", delta / 86400)
    }
}
