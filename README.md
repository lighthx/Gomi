# Gomi

A minimalist browser launcher for macOS that helps you manage multiple browsers and their profiles. Gomi lets you quickly switch between browsers and profiles based on URLs.

## Features

- 🚀 Fast browser switching with a clean interface
- 🎯 URL pattern matching (exact/contains)
- 👤 Browser profile management
- ⌨️ Keyboard shortcuts support
- 🔄 Automatic browser/profile selection based on URL patterns

## Installation

```bash
# Using Cargo
git clone https://github.com/lighthx/gomi

cargo install cargo-bundle

cargo bundle --release

# Or download from releases
https://github.com/lighthx/gomi/releases
```

## Usage

### Basic Usage
1. Set Gomi as your default browser
2. Click any link to activate Gomi
3. Select your preferred browser/profile
4. (Optional) Save your choice for similar URLs

### Keyboard Shortcuts
- `⌘` + Click: Save browser choice for exact URL match
- `⇧` + Click: Create custom URL pattern

### URL Pattern Matching
Gomi supports two types of URL matching:
- Exact Match: Matches the complete URL
- Contains Match: Matches part of the URL

### Browser Profiles
Each browser can have multiple profiles:
- Click the profile icon next to a browser
- Add/Remove profiles as needed
- Set different URL patterns for different profiles

## Development

```bash
# Clone the repository
git clone https://github.com/lighthx/gomi
cd gomi

# Build
cargo build --release

# Run tests
cargo test
```

## Requirements
- macOS 10.15+
- Rust 1.70+

## License
MIT

## TODO

- [ ] Add a management page for matched URLs; currently, deleting/editing matched URLs is not supported.
- [ ] Improve the ugly UI

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.
