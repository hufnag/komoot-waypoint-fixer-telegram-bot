# Komoot Waypoint Fixer — Telegram bot

## Why I started this project

I started this project because waypoints shown on Gen3 Wahoo ELEMNT devices are corrupted during transfer from Komoot. I already contacted the Komoot support about the issue and am still waiting for a fix. So hopefully this tool won't be necessary anymore once they resolve the problem.

## How it works

The tool is a small Telegram bot that helps you fixing the waypoints in your Komoot tours. After sending the exported GPX file to the bot, it will walk through each waypoint and present an inline keyboard to choose the waypoint type. After all waypoints are processed the bot sends back a fixed GPX file which can then be transfered to the Wahoo App. The transfer of the file from Komoot to the Telegram bot and then to the Wahoo App works conveniently via the Share menu on iOS and hopefully on Android as well.

Step by step guide:
1. Open the planned tour in the Komoot app and share the GPX file via the "Download GPX" option to your Telegram Bot
2. Answer the inline keyboard questions to fix the waypoints
3. Share the received fixed GPX file to the Wahoo App


## Prerequisites

- Rust toolchain (see https://rust-lang.org/tools/install/)
- A Telegram account

## Setup

1. Setup your own Telegram bot via the [BotFather](https://t.me/BotFather)
2. Set the `TELOXIDE_TOKEN` environment variable to your bot token from step 1 (e.g. `export TELOXIDE_TOKEN="<your-telegram-bot-token>"`)
3. Run the bot using `cargo run --release`

As an alternative to running the bot on your local machine, you can also set it up as a systemd service e.g. on a Raspberry Pi:

1. Cross compile the bot for your target architecture using cross: e.g. `cross build --release --target aarch64-unknown-linux-gnu` for a Raspberry Pi 4
2. Copy the compiled binary from `target/aarch64-unknown-linux-gnu/release/komoot-waypoint-fixer-telegram-bot` to `/usr/local/bin` on your target machine.
3. Set your Telegram bot token in  `komoot-waypoint-fixer-telegram-bot.service` and copy it to `/etc/systemd/system` on your target machine.
4. Enable and start the service: `sudo systemctl enable --now komoot-waypoint-fixer-telegram-bot.service`

## Contributing

Feel free to open issues or PRs.
