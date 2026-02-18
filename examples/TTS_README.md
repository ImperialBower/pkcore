# Text-to-Speech Examples

This directory contains examples that demonstrate using text-to-speech (TTS) to make your poker applications speak!

## Library Used

We use the [`tts`](https://crates.io/crates/tts) crate, which provides cross-platform text-to-speech:

- **macOS**: Uses AVFoundation (built-in)
- **Windows**: Uses SAPI/Speech Platform (built-in)
- **Linux**: Uses Speech Dispatcher (may need installation)

## Installation

### macOS
No additional setup needed! Just run the examples.

### Linux
You may need to install Speech Dispatcher:

```bash
# Ubuntu/Debian
sudo apt-get install speech-dispatcher libspeechd-dev

# Fedora
sudo dnf install speech-dispatcher speech-dispatcher-devel

# Arch
sudo pacman -S speech-dispatcher
```

### Windows
No additional setup needed on Windows 10/11.

## Available Examples

### 1. Game State Demo with Speech
Basic example showing how to speak game state information.

```bash
cargo run --example game_state_demo_with_speech
```

This example:
- Announces the table setup
- Speaks blind amounts
- Narrates game phase changes
- Reports pot sizes and betting action

### 2. Poker Narrator
Advanced example that narrates a full poker hand.

```bash
cargo run --example poker_narrator
```

This example:
- Provides detailed game narration
- Announces each phase of play
- Speaks player actions
- Reports deck status and active players

## Features

The TTS integration provides:

- **Real-time announcements**: Speak game events as they happen
- **Customizable speech**: Control rate, pitch, and volume
- **Accessible gaming**: Make poker games accessible to visually impaired users
- **Enhanced UX**: Add voice feedback to your poker applications

## API Usage

Basic usage pattern:

```rust
use tts::Tts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize TTS engine
    let mut tts = Tts::default()?;
    
    // Speak text (non-blocking)
    tts.speak("Your text here", false)?;
    
    // Or speak and wait for completion
    tts.speak("Your text here", true)?;
    
    Ok(())
}
```

## Advanced Configuration

You can configure speech properties:

```rust
// Set speaking rate (0.0 - 10.0, default varies by platform)
tts.set_rate(1.5)?;

// Set volume (0.0 - 1.0)
tts.set_volume(0.8)?;

// Set pitch (0.0 - 2.0, 1.0 is normal)
tts.set_pitch(1.2)?;

// Get available voices
let voices = tts.voices()?;

// Set a specific voice
if let Some(voice) = voices.first() {
    tts.set_voice(&voice)?;
}
```

## Use Cases

Perfect for:

- **Accessibility**: Screen reader integration for poker software
- **Training tools**: Announce hand ranges, odds, and decisions
- **Live games**: Dealer-style commentary
- **Debugging**: Audio feedback during development
- **Multi-tasking**: Listen to game status without looking at screen

## Tips

1. **Keep messages concise**: TTS works best with short, clear sentences
2. **Avoid symbols**: Replace symbols with words (e.g., "♠" → "spades")
3. **Add pauses**: Use `std::thread::sleep()` between announcements
4. **Test voices**: Different voices may sound better for your use case
5. **Handle errors**: TTS may not be available on all systems

## Troubleshooting

**No sound on Linux?**
- Make sure Speech Dispatcher is running: `systemctl --user start speech-dispatcher`
- Test with: `spd-say "Hello world"`

**Speech too fast/slow?**
- Adjust the rate: `tts.set_rate(0.8)?;`

**Can't hear anything?**
- Check system volume
- Verify TTS is initialized: `let mut tts = Tts::default()?;`
- Try synchronous speech: `tts.speak("test", true)?;`

## Further Reading

- [tts crate documentation](https://docs.rs/tts/)
- [Text-to-Speech on Wikipedia](https://en.wikipedia.org/wiki/Speech_synthesis)
- [Accessibility guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

