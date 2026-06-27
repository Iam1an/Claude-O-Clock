# claudeoclock

> *youtube time is over*

A floating desktop agent monitor for Claude Code. Claudeoclock watches your Claude agents while you do literally anything else — then wakes you up when they finish, error out, or need your input.

Built with Rust + Tauri + React. Runs on Arch Linux (Hyprland/Wayland). Also works on macOS.

---

## What it does

You spawn a Claude Code agent on a task and walk away. Maybe you're watching something, cooking, or working on something else entirely. Claudeoclock sits as a floating widget on your desktop and monitors all your running agents in real time. The moment something needs your attention — an agent finishes, hits an error, or is waiting on you — your computer makes a sound and your phone buzzes.

No more tabbing back every five minutes to check. No more agents sitting idle because they finished an hour ago and you didn't notice.

---

## Features

- **Floating widget** — always-on-top native window, sits in the corner of your Hyprland setup while you work
- **Liquid glass UI** — dark minimal aesthetic with Claude orange accents, smooth animations, custom vector icons
- **Live agent grid** — up to 10 agents as cards, single column up to 5, two columns at 6–10
- **6 agent states** — Thinking, Planning, Doing, Compacting, Done, Error — each with a custom icon
- **Sound alerts** — distinct sounds for Done (ding) and Waiting/Error (urgent alert), all swappable
- **Phone notifications** — ntfy by default, Telegram and OpenClaw SMS as optional plugins
- **Per-agent alarms** — set an alarm per agent at spawn time, with a global Do Not Disturb mode
- **Spawn agents from the UI** — launch new Claude agents without leaving Claudeoclock
- **Auto-detect existing agents** — attaches to already-running Claude Code processes automatically
- **Kill and restart** — stop a runaway agent and restart it with a new prompt from the UI
- **Full MCP server** — exposes 11 tools so Claude Code, OpenClaw, and other MCP clients can query and control Claudeoclock programmatically
- **SQLite persistence** — full agent history survives restarts, crashes, and reboots
- **TOML config** — human-readable config at `~/.config/claudeoclock/config.toml`, editable visually from the Settings tab
- **First-run wizard** — guided setup on first launch to configure notifications, sounds, and Claude hooks
- **Arch Linux native** — built for Hyprland/Wayland, auto-launches as floating window

---

## Installation

### Option 1 — GitHub Releases (recommended)

Download the latest pre-built binary from the [Releases](https://github.com/yourusername/claudeoclock/releases) page.

```bash
# Arch Linux
curl -L https://github.com/yourusername/claudeoclock/releases/latest/download/claudeoclock-x86_64-linux.AppImage -o claudeoclock
chmod +x claudeoclock
sudo mv claudeoclock /usr/local/bin/
```

```bash
# macOS (Apple Silicon)
curl -L https://github.com/yourusername/claudeoclock/releases/latest/download/claudeoclock-aarch64.dmg -o claudeoclock.dmg
open claudeoclock.dmg
```

### Option 2 — Build from source

Requires Rust and Node.js installed.

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js on Arch
sudo pacman -S nodejs npm

# Install Tauri CLI
cargo install tauri-cli

# Clone and build
git clone https://github.com/yourusername/claudeoclock.git
cd claudeoclock
npm install
cargo tauri build
```

For faster Rust compile times on Arch, install `sccache` first:

```bash
sudo pacman -S sccache
export RUSTC_WRAPPER=sccache
```

---

## Quick Start

```bash
claudeoclock
```

On first launch, the setup wizard walks you through:
- Configuring your notification channel (ntfy topic, Telegram token, or OpenClaw webhook)
- Setting your alert sounds (or keeping the defaults)
- Pointing Claudeoclock at your Claude Code hooks directory

After setup, Claudeoclock auto-detects any running Claude Code agents and populates the grid immediately. The window launches floating and always-on-top automatically on Hyprland.

### Hyprland window rule (optional — add to your hyprland.conf)

```
windowrulev2 = float, class:claudeoclock
windowrulev2 = pin, class:claudeoclock
windowrulev2 = size 420 600, class:claudeoclock
windowrulev2 = move 100%-440 40, class:claudeoclock
```

This pins Claudeoclock to the top-right corner of your screen across all workspaces.

---

## UI Overview

Claudeoclock is a floating native window with four tabs.

### Overview Tab
Summary stats at a glance:
- Total agents running
- Agents by status (how many Thinking, Doing, Done, etc.)
- Agents needing attention (Waiting or Error)
- Uptime and session info

### Agents Tab
The main grid. Each agent card shows:
- **Agent name** — set at spawn time
- **Status icon** — custom vector icon per state
- **Elapsed time** — how long the agent has been running
- **Current task** — what the agent is working on right now
- **Token count** — tokens used so far

Click any card to open the **fullscreen agent detail view**, which adds:
- Current tool being used
- Last action taken
- Full log output
- Alarm settings for that agent

### Alerts Tab
History of all alerts fired — what agent, what state, what time. Clear individual alerts or the full history.

### Settings Tab
Visual editor for `~/.config/claudeoclock/config.toml`. Changes save immediately to disk. Covers:
- Notification channels and credentials
- Sound file paths (swap in your own sounds)
- Global Do Not Disturb toggle and schedule
- Claude hooks directory path
- Agent limit (default 10)

---

## Agent States

| Icon | State | Meaning |
|---|---|---|
| 🟣 | Thinking | Claude is reasoning through the problem |
| 🔵 | Planning | Claude is building a plan of action |
| 🟠 | Doing | Claude is actively executing tasks |
| 🟡 | Compacting | Context window is being compressed |
| 🟢 | Done | Task complete — alarm fires |
| 🔴 | Error | Something went wrong — alarm fires |
| ⚪ | Waiting | Claude needs your input — alarm fires |

---

## Alerts and Notifications

### Sound Alerts
Two sounds ship with Claudeoclock:

- **Done sound** — a satisfying bell/ding. Plays when an agent reaches Done.
- **Urgent sound** — a harsh alert. Plays when an agent hits Error or Waiting.

Swap in your own sounds via the Settings tab or config file:

```toml
[sounds]
done = "/home/user/.config/claudeoclock/sounds/done.ogg"
urgent = "/home/user/.config/claudeoclock/sounds/urgent.ogg"
```

Claudeoclock uses `paplay` (PulseAudio/PipeWire) on Linux and `afplay` on macOS. Both are pre-installed on standard Arch and macOS setups.

### Phone Notifications

#### ntfy (default — free, self-hosted)
1. Install the [ntfy app](https://ntfy.sh) on your phone (Android or iOS)
2. Add your topic to config:

```toml
[notifications]
provider = "ntfy"
ntfy_url = "https://ntfy.sh"
ntfy_topic = "claudeoclock-your-secret-topic"
```

3. Subscribe to the same topic in the ntfy app. Done.

#### Telegram (optional plugin)
1. Message [@BotFather](https://t.me/botfather) on Telegram, create a bot, copy the token
2. Get your chat ID from [@userinfobot](https://t.me/userinfobot)
3. Add to config:

```toml
[notifications]
provider = "telegram"
telegram_token = "your-bot-token"
telegram_chat_id = "your-chat-id"
```

#### OpenClaw SMS (optional plugin)
For users already running OpenClaw. Sends a real SMS to your phone number.

```toml
[notifications]
provider = "openclaw"
openclaw_webhook = "http://localhost:PORT/your-webhook"
```

---

## Per-Agent Alarms

When spawning a new agent, you set whether you want to be alerted when it changes state:

```
Spawn new agent
  Name: refactor-auth
  Prompt: Refactor the auth module to use JWT...
  Working dir: ~/projects/myapp
  Alarm: [x] Done  [x] Error  [x] Waiting
```

Uncheck any event you don't care about for that specific agent. A throwaway test agent doesn't need to wake you up.

### Global Do Not Disturb
Toggle DND from the Settings tab or the top bar of the UI to silence all sounds and phone notifications. All alerts are still logged in the Alerts tab — just not delivered until DND is off.

Schedule DND automatically in config:

```toml
[dnd]
enabled = false
start = "23:00"
end = "08:00"
```

---

## MCP Server

Claudeoclock runs a local MCP server on startup (default port `22361`), exposing 11 tools to any MCP-compatible client — including Claude Code itself, OpenClaw, and custom scripts.

### Connecting Claude Code to Claudeoclock

Add to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "claudeoclock": {
      "command": "claudeoclock",
      "args": ["--mcp"]
    }
  }
}
```

Then from inside Claude Code you can say things like:
- *"What are my agents doing?"*
- *"Kill agent 3 and restart it"*
- *"Mute alarms for agent 5"*
- *"Spawn a new agent to write tests for auth.rs"*

### Available MCP Tools

| Tool | Description |
|---|---|
| `list_agents` | Returns all agents and their current status |
| `get_agent` | Returns full detail for a specific agent |
| `spawn_agent` | Starts a new Claude Code agent |
| `kill_agent` | Terminates a running agent |
| `restart_agent` | Kills and restarts an agent with the same or new prompt |
| `get_status` | Returns overall system status and stats |
| `set_alarm` | Configure alarm settings for a specific agent |
| `mute_agent` | Mute/unmute notifications for a specific agent |
| `get_logs` | Returns log output for a specific agent |
| `dnd_mode` | Enable or disable global Do Not Disturb |
| `get_overview_stats` | Returns summary stats (total, by state, needing attention) |

---

## Configuration

Full config reference for `~/.config/claudeoclock/config.toml`:

```toml
[general]
max_agents = 10
claude_hooks_dir = "~/.claude/hooks"
refresh_rate_ms = 500

[sounds]
enabled = true
done = "built-in"       # or path to .ogg/.wav file
urgent = "built-in"     # or path to .ogg/.wav file

[notifications]
provider = "ntfy"       # ntfy | telegram | openclaw | none
ntfy_url = "https://ntfy.sh"
ntfy_topic = ""         # set this to your secret topic
telegram_token = ""
telegram_chat_id = ""
openclaw_webhook = ""

[alerts]
on_done = true
on_error = true
on_waiting = true

[dnd]
enabled = false
start = "23:00"
end = "08:00"

[mcp]
enabled = true
port = 22361

[ui]
theme = "dark"
accent = "#E67E22"      # Claude orange — change if you want
```

---

## How It Works

Claudeoclock integrates with Claude Code's native hook system. When Claude Code fires a hook event (tool call, status change, completion), the Rust backend receives it, updates agent state in SQLite, pushes the update to the React frontend via Tauri's event system, and fires any configured alerts.

```
Claude Code agent
    → fires hook event
        → Rust backend (Tauri)
            → updates SQLite
            → pushes event to React frontend
            → refreshes agent card in UI
            → fires sound alert (if configured)
            → sends phone notification (if configured)
            → available via MCP tools
```

No proxying, no wrapping the Claude binary, no fragile process inspection. Hook-based only — the same approach used by the best Claude Code integrations.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust + TypeScript |
| Desktop framework | Tauri v2 |
| Frontend | React + TypeScript |
| Styling | CSS — liquid glass, Claude orange |
| Database | SQLite via `rusqlite` |
| MCP server | `rmcp` crate |
| Config | TOML via `toml` crate |
| Sound (Linux) | `paplay` (PipeWire/PulseAudio) |
| Sound (macOS) | `afplay` |
| Notifications | ntfy / Telegram Bot API / OpenClaw webhook |
| Build | Cargo + sccache + Tauri CLI |

---

## Platform Support

| Platform | Status |
|---|---|
| Arch Linux (Hyprland/Wayland) | ✅ Primary target |
| Arch Linux (X11) | ✅ Supported |
| macOS (Apple Silicon) | ✅ Supported |
| macOS (Intel) | ✅ Supported |
| Other Linux distros | 🟡 Should work, untested |
| Windows | ❌ Not planned |

---

## Roadmap

- [ ] Core Tauri app with React frontend
- [ ] Liquid glass UI with Claude orange theme
- [ ] Claude Code hook integration
- [ ] Agent grid with card layout
- [ ] Fullscreen agent detail view
- [ ] Sound alerts (Done + Urgent)
- [ ] ntfy phone notifications
- [ ] Per-agent alarm configuration
- [ ] Global DND mode
- [ ] SQLite persistence
- [ ] Spawn and kill agents from UI
- [ ] Auto-detect existing Claude Code processes
- [ ] MCP server with 11 tools
- [ ] First-run setup wizard
- [ ] TOML config + Settings tab visual editor
- [ ] Hyprland window rules documentation
- [ ] Telegram notification plugin
- [ ] OpenClaw SMS plugin
- [ ] Custom sound file support
- [ ] GitHub Releases CI pipeline (Arch + macOS binaries)
- [ ] Compact/expanded window modes
- [ ] Pause/resume agents (pending Claude Code support)
- [ ] Team/multi-user features
- [ ] Hosted version

---

## Contributing

Pull requests welcome. Open an issue first for anything significant so we can discuss before you build.

```bash
git clone https://github.com/yourusername/claudeoclock.git
cd claudeoclock
npm install
cargo tauri dev
```

---

## License

MIT — free to use, modify, and distribute. If this gets traction, a commercial license tier may be introduced for enterprise features while the core remains MIT.

---

*Built on Arch. Tested on patience.*
