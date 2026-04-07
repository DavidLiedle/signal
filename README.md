# signal

A tiny CLI that maps how you're feeling to something you can actually do about it.

Type a mood, get a suggestion. That's it.

```
$ signal stuck
Describe the problem out loud as if explaining it to a friend.
```

## Install

```
cargo install --git https://github.com/DavidLiedle/signal
```

Or build from source:

```
git clone https://github.com/DavidLiedle/signal
cd signal
cargo build --release
cp target/release/signal ~/.local/bin/   # or wherever you keep binaries
```

## Usage

```
signal                       Random suggestion (biased toward least-recently shown)
signal --mood <mood>         Random suggestion within a specific mood
signal <mood>                Show all suggestions for a mood
signal add <mood> <msg>      Add a suggestion to a mood
signal remove <mood>         Remove an entire mood
signal remove <mood> <msg>   Remove a specific suggestion
signal list                  List all moods and suggestions
signal path                  Print the data file location
```

### Quick examples

```
$ signal
Go somewhere public — a cafe, a park — just to be around people.

$ signal anxious
Step outside for 5 minutes and breathe slowly.
Name three things you can see, hear, and feel right now.

$ signal --mood tired
Drink a full glass of water — dehydration feels a lot like fatigue.

$ signal add frustrated Take a break and come back with fresh eyes.
Added to: frustrated

$ signal list
  anxious:
    - Step outside for 5 minutes and breathe slowly.
    - Name three things you can see, hear, and feel right now.
  frustrated:
    - Take a break and come back with fresh eyes.
  ...
```

## How it works

Suggestions are stored in a JSON file at `~/.config/signal/signals.json` (macOS/Linux). The file is created automatically on first run with a set of defaults covering 9 moods, 2 suggestions each.

Each suggestion tracks when it was last shown. When you run `signal` with no arguments (or with `--mood`), selection is weighted toward suggestions you haven't seen recently — so you get variety without pure randomness.

You can override the data file location with the `SIGNAL_DATA` environment variable:

```
SIGNAL_DATA=~/my-signals.json signal
```

## Default moods

| Mood | Example suggestion |
|------|-------------------|
| anxious | Step outside for 5 minutes and breathe slowly. |
| bored | Pick one small thing you've been putting off and do just the first step. |
| distracted | Close all tabs, pick one task, set a 25-minute timer. |
| lonely | Text one person you haven't talked to in a while. |
| overwhelmed | Write down everything on your mind, then pick just one item. |
| restless | Go for a short walk — even 10 minutes resets your head. |
| sad | Put on a song you love and let yourself feel it. |
| stuck | Describe the problem out loud as if explaining it to a friend. |
| tired | Take a 20-minute nap or splash cold water on your face. |

All moods are fully customizable. Add your own, remove the defaults, make it yours.

## License

MIT
