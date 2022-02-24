<p align="center">
  <img alt="Mokuroku" src="./docs/logo.png" width="25%">
</p>

<p align="center">
  A hassle-free, highly performant, host-it-yourself Discord anime watch-list bot
</p>

## Deployment

### Usage
Make a copy of `.env.example` and name it `.env`.

Then [create a bot account](https://discord.com/developers/applications), and copy its **token** to `.env` with the `DISCORD_TOKEN` environment.

You also need to enter your [MongoDB](https://mongodb.com/) cluster info into the `.env` file.

### Docker

```shell
docker run -d --env-file .env ghcr.io/Shorakie/mokuroku:latest
```

## Development

Make sure you've installed Rust. You can install Rust and its package manager, `cargo` by following the instructions on https://rustup.rs/.
Simply run `cargo run`.

## Testing

Tests are available inside the `src/tests` folder. They can be ran via `cargo test`. It's recommended that you run the tests before submitting your Pull Request.
Increasing the test coverage is also welcome.

### Docker

Within the project folder, simply run the following:

```shell
docker build -t mokuroku .
docker run -d --env-file .env mokuroku
```