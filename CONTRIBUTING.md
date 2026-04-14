# Contributing to ave-cloud-rs-skills

Thank you for your interest in contributing to ave-cloud-rs-skills.

## Development Setup

1. Install Rust (latest stable)
2. Clone the repository
3. Run `cargo build` to verify setup

## Running Tests

```bash
cargo test
cargo test --features rustls  # with rustls
```

## Code Quality

```bash
cargo fmt      # format code
cargo clippy   # lint
```

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `refactor:` Code refactoring
- `test:` Adding tests
- `chore:` Maintenance

## Adding a New Skill

1. Create `skills/<skill-name>/SKILL.md`
2. Follow the SKILL.md template from existing skills
3. Implement the skill in `src/`
4. Add CLI subcommand in `src/main.rs`

## Security

- Never log credentials or private keys
- Never commit `.env` files
- Report security issues via GitHub Security tab

## Pull Request Checklist

- [ ] `cargo check` passes
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt` applied
- [ ] Documentation updated if needed
