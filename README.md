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
  # ahead_or_behind_the_beat_ratio_slider is optional
  ahead_or_behind_the_beat_ratio_slider:
    channel: 1
    control_change: 2
  # randomize_note_start_time_ratio_slider is optional
  randomize_note_start_time_ratio_slider:
    channel: 1
    control_change: 3
# progression is optional, defaults to staying in the key of C
progression: C C C C Eb Eb Eb Eb
```

```shell
cargo run -- config.yml
```

