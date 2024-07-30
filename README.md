# Hedon Bot 🤖

Hedon Bot is a Feishu bot designed to send scheduled updates on the latest news and developments in Go and Rust programming languages. It fetches information from various channels to keep you updated with the most recent trends and changes in these technologies.



## Features 🌟

- Periodically fetches articles from [Golang Weekly](https://golangweekly.com/) and sends updates to designated Feishu groups via the bot.
- Periodically fetches articles from [Go Official Blog](https://go.dev/blog/) and sends updates to designated Feishu groups via the bot.
- Periodically fetches articles from [Redis Official Blog](https://redis.io/blog/) and sends updates to designated Feishu groups via the bot.
- Uses ChatGPT to summarize the article content.

## Used Rss/Atom Feeds

- [Golang Weekly](https://cprss.s3.amazonaws.com/golangweekly.com.xml)
- [Go Official Blog](https://go.dev/blog/feed.atom)
- [Redis Official Blog](https://redis.io/blog/feed/)

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

   Copy the `config.template.yml` to `config.yml` and modify it as needed.

4. **Run Hedon-Bot in the background**
   ```bash
   hedon-bot&
   ```
   Make sure `config.yml` is in the current directory (command line arguments specifying the configuration file path will be supported later. Please look forward ~)



## Configuration (`config.yml`) ⚙️

- **openai_api_key**: The api key to invoke OpenAI api (optional), you can get it from `https://platform.openai.com/api-keys`.
- **openai_host**: The OpenAI api host (optional), if your server environment or area does not support access to the openai website, you need to configure it.
- **proxy**: The proxy address (optional), used for proxy access to the Open API, if your server environment does not support access to the corresponding website, you need to configure it.
- **redis**: Configuration for Redis connection. This project uses Redis's `hsetnx` to prevent pushing the same article more than once.
- **webhooks**: Specify the list of webhooks for various channels. Currently supports the `go_weekly`, `go_blog` and `redis_official_blog`, with support for multiple webhooks per channel.
- **cron_expression**: Schedule the frequency of fetching updates for each channel using a cron expression format:
   ```
   sec   min   hour   day of month   month   day of week   year
   *     *     *      *              *       *             *
   ```
- **once_post_limit**: A limit on the number of articles it can push at a time, default is `5`.



## Core Crates 📚

- **tokio-cron-scheduler**: For scheduling tasks in async.
- **quick-xml**: XML parsing library.
- **serde_yml**: YAML parsing library.
- **serde_json**: JSON parsing library.
- **scraper**: HTML manipulation library.
- **reqwest**: HTTP client.
- **redis**: Redis library.
- **tracing**: Tracing log.
- **tracing**: Provides structured logging and event tracing for Rust applications.
- **tracing_subscriber**: Implements log subscribers to handle and format tracing data.
- **tracing_appender**: Manages log output destinations, supporting asynchronous logging and log file rotation.



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
