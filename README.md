# Eureka notify

A discord bot that periodically tracks upcoming lucrative windows for Eureka. It outputs the state as a Discord message in the channel of your choosing.

![example output](https://i.imgur.com/nFXMCRi.png)

---
## Installation

Create your bot at the [Discord Developer Portal](https://discord.com/developers/applications)

Create a `.env` file with the following keys and fill them out:

    DISCORD_TOKEN=
    CHANNEL_ID=

Add your discord token:
  - From the [Developer Portal](https://discord.com/developers/applications), open the `Bot` menu
  - Click the "Reset Token" button
  - Paste the generated code after "DISCORD_TOKEN=" in your `.env` file

Add your bot to your server of choice:
  - From the [Developer Portal](https://discord.com/developers/applications), open the `OAuth2` menu > `URL Generator`
  - Check the "Bot" scope
  - Check the "Send Messages" text permission
  - Navigate to the generated URL at the bottom
  - Select the server to add your bot to, and add i

Get your channel ID:
  - [Enable developer mode](https://www.howtogeek.com/714348/how-to-enable-or-disable-developer-mode-on-discord/) if you haven't yet already
  - Right-click the channel you want this bot to output to > `Copy ID`
  - Paste the code after "CHANNEL_ID=" in your `.env` file

Compile the bot:
  - [Download and install Rust](https://www.rust-lang.org/tools/install)
  - Run your bot with the command: `cargo run`
  - Build a standalone executable with the commmand: `cargo build --release`
    - The generated executable will be located in `target/release`