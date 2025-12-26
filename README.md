# Centrum Launcher

An application launcher for Linux, built with Rust and GTK4.

Centrum Launcher is my very first project which I used to learn/practice coding, I designed it to be a center tool for my PC experience hence why i tried to build a lot of tools into it.
Since originally I never meant to release it and it was just a learning project, I used a lot of AI help (since as this was my first ever project of any kind) I felt important to disclose it.
But over time I thought it did become a really good "centrum" for quick needs, I decided to try release it even if it's not something i'm proud of.![screenrecording_2025-12-26_16-53](https://github.com/user-attachments/assets/12fc1da9-1417-415e-aa97-0d42572f325e) 
![CentrumLauncher](https://github.com/user-attachments/assets/9f7e3095-549e-40c4-8cc7-1b66dee1f699)

## Features

- **DE-Agnostic & Modular**: Built-in native support for Niri and Hyprland (including window switching/focus logic), with generic Wayland support for all other compositors.
- **Fuzzy Matching**: Rapidly find and launch applications.
- **Focus-on-Launch**: If a program is already running, Centrum will switch focus to it instead of launching a duplicate (supported on Niri and Hyprland, toggleable in config).
- **File Search**: Search your home directory by starting your query with `/` (requires `fd`), open files straight from the launcher or open folders in your default file manager.
- **Web Search**: Configurable search engine support (Google, Startpage, DuckDuckGo, Bing, Ecosia, Qwant). Use `?` to search.
- **Website opening**: just type in the website you wanna open like `google.com` and it opens in your default browser.
- **Calculator**: Integrated quick calculations using `qalc`. Just type in equations and conversions like `100m to yd`, `1 eur to usd`
- **Clipboard History**: Access and filter your clipboard history (requires `cliphist`).
- **Color Picker**: Built-in color picker with HEX and RGB copy support.
- **Customizable**: Override program names/icons (using the hotkeys), hide programs, and customize themes, search engines, and hotkeys via CSS.
- **Power Menu**: Integrated power options (Shutdown, Reboot, Logout) that can be expanded by your need, for example you add a theme switcher.

## Requirements

- `gtk4`
- `fd` (for file search)
- `libqalculate` (for `qalc` math support)
- `cliphist` (for clipboard history)
- `wl-copy` (for Wayland clipboard support)
- `niri` or `hyprland` (optional, for advanced window management features)

## Installation

1. Clone the repository:
   
`git clone https://github.com/Kythreaa/centrum-launcher.git
   cd centrum-launcher`

3. Build the project:
   
` cargo build --release `

4. Copy the binary to your path:

`
   cp target/release/centrum-launcher ~/.local/bin/
`

## Configuration

Centrum Launcher generates its default configuration files in `~/.config/centrum-launcher/` on the first run:
- `config.css`: Main configuration for UI, animations, hotkeys, search engines, and power commands.
- `light.css` / `dark.css`: Theme-specific color definitions.

## License

This project is licensed under the GNU General Public License v3.0
