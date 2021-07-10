## Setup

Install [`rustup`](https://rustup.rs/). Install the [`rust-analyzer`](https://rust-analyzer.github.io) language server if desired.

## Formatting

```shell
cargo fmt
```

## Linting

```shell
cargo clippy
```

## Tests

```shell
cargo test
```

## Running

Create a configuration file:

```yaml
# midi is optional
midi:
  port: # put any string here to have line-runner show you a list of available ports
  # duration_ratio_slider is optional
  duration_ratio_slider:
    channel: 1
    control_change: 1 # modulation wheel
```

```shell
cargo run -- config.yml
```

