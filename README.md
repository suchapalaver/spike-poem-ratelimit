# Spike `poem-ratelimit`!

Uses a fork of `poem-ratelimit` that uses the latest version of `poem`:

```toml
[dependencies]
poem-ratelimit = { git = "https://github.com/suchapalaver/poem-ratelimit.git" }
```

Uses [`testcontainers`](https://github.com/testcontainers/testcontainers-rs)
to spin up a Redis instance for testing.
