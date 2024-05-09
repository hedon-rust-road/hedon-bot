# Hedon Bot 🤖

Hedon Bot is a Feishu bot designed to send scheduled updates on the latest news and developments in Go and Rust programming languages. It fetches information from various sources to keep you updated with the most recent trends and changes in these technologies.



## Features 🌟

- Periodically fetches articles from [Golang Weekly](https://golangweekly.com/) and sends updates to designated Feishu groups via the bot.



## Installation 🔧

Follow these steps to install and set up Hedon-Bot:

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

2. **Install Hedon-Bot**
   ```bash
   cargo install --git https://github.com/hedon-rust-road/hedon-bot
   ```

3. **Configure Hedon-Bot**

   Copy the `config.template.yml` to `config.yml` and modify it as needed:
   ```bash
   cp config.template.yml config.yml
   ```

4. **Modify the `log4rs.yml` log configuration file as necessary**



## Configuration (`config.yml`) ⚙️

- **redis**: Configuration for Redis connection. This project uses Redis's `hsetnx` to prevent pushing the same article more than once.
- **webhook**: Specify the list of webhooks for various channels. Currently supports the `go_weekly` channel, with support for multiple webhooks per channel.
- **cron_expression**: Schedule the frequency of fetching updates for each channel using a cron expression format:
   ```
   sec   min   hour   day of month   month   day of week   year
   *     *     *      *              *       *             *
   ```



## Core Frameworks and Libraries 📚

- **job_scheduler**: For scheduling tasks.
- **log4rs**: Logging library.
- **quick-xml**: XML parsing library.
- **serde_yml**: YAML parsing library.
- **serde_json**: JSON parsing library.
- **scraper**: HTML manipulation library.
- **reqwest**: HTTP client.
- **redis**: Redis library.



## Planned Features 🚀

1. Support for more channels of high-quality Go articles.
2. Extend support to the Rust programming language.



## Contributing 🤝

Contributions are welcome! If you're looking to contribute to Hedon-Bot, please follow the steps below to set up your development environment.

### Install Rust

Start by installing Rust if you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install Visual Studio Code Plugins

To enhance your development experience, install the following VSCode plugins, you can find all these plugins in `hedon-rust-pack` in vscode extension market:

- **crates**: For Rust package management.
- **Even Better TOML**: For enhanced TOML file support.
- **Better Comments**: Improves comment visibility.
- **Error Lens**: Enhances error highlighting.
- **GitLens**: Provides Git superpowers.
- **GitHub Copilot**: Offers code suggestions.
- **indent-rainbow**: Makes indentation more readable.
- **Prettier - Code formatter**: Automates code formatting.
- **REST client**: Useful for debugging REST APIs.
- **rust-analyzer**: Essential for Rust language support.
- **Rust Test Lens**: Adds in-line Rust test running.
- **Rust Test Explorer**: Provides a test suite overview.
- **TODO Highlight**: Highlights TODOs in your code.
- **vscode-icons**: Enhances file explorer with rich icons.
- **YAML**: Adds support for YAML files.

### Install cargo generate

`cargo generate` helps you start new projects using templates:

```bash
cargo install cargo-generate
```

This project uses the `hedon-rust-road/template` template to generate the basic code structure:

```bash
cargo generate --git https://github.com/hedon-rust-road/template
```

### Install pre-commit

`pre-commit` is a tool to run checks on your code before committing. It ensures you're not committing anything that doesn't meet the project's coding standards:

```bash
pipx install pre-commit
pre-commit install
```

### Install Cargo deny

`Cargo deny` checks your dependencies for security vulnerabilities, licensing issues, and more:

```bash
cargo install --locked cargo-deny
```

### Install typos

`typos` is a command-line tool to check your code for spelling errors:

```bash
cargo install typos-cli
```

### Install git cliff

`git cliff` generates changelogs based on the history of your Git repository:

```bash
cargo install git-cliff
```

### Install cargo nextest

`cargo nextest` is a modern test runner for Rust that provides better test output and controls:

```bash
cargo install cargo-nextest --locked
```

### Making Changes

Once you have your environment set up, you are ready to contribute to Hedon-Bot. Here’s what you need to know about making changes:

- **Fork and clone the repo**: Start by forking the repository and then cloning it locally.
- **Install pre-commit**: Enter the project root directory and install pre-commit tool by run `pre-commit install`.
- **Create a branch**: Make a branch for your changes. It helps isolate your changes and keeps the main branch free of unstable code.
- **Make your changes**: Implement your changes or improvements.
- **Run tests and lint checks**: Before submitting your changes, make sure all tests pass and all lint checks are satisfied.
- **Submit a pull request**: Push your changes to your fork and then submit a pull request. It will be reviewed as soon as possible.

Thank you for contributing to Hedon-Bot! We look forward to seeing your contributions.
