# Contributing to Zeta Reticula

We're thrilled you're interested in contributing to Zeta Reticula! This guide will help you get started with the contribution process.

## 🛠 Development Setup

1. **Fork the Repository**
   - Click the "Fork" button in the top-right corner
   - Clone your fork: `git clone https://github.com/your-username/zeta-reticula.git`
   - Add upstream: `git remote add upstream https://github.com/your-org/zeta-reticula.git`

2. **Set Up Environment**
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install Node.js (for frontend development)
   # Using nvm (recommended)
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   nvm install --lts
   
   # Install dependencies
   cargo install cargo-watch
   npm install -g pnpm
   ```

3. **Build and Test**
   ```bash
   # Build the project
   cargo build
   
   # Run tests
   cargo test
   
   # Run lints
   cargo clippy --all-targets --all-features -- -D warnings
   ```

## 🚀 Making Changes

1. **Create a Branch**
   ```bash
   git checkout -b feat/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Commit Your Changes**
   - Follow [Conventional Commits](https://www.conventionalcommits.org/)
   - Write clear, concise commit messages
   - Keep commits focused and atomic

3. **Run Tests**
   ```bash
   cargo test
   cargo clippy
   ```

4. **Push Your Changes**
   ```bash
   git push origin your-branch-name
   ```

5. **Open a Pull Request**
   - Go to the [Pull Requests](https://github.com/your-org/zeta-reticula/pulls) page
   - Click "New Pull Request"
   - Fill in the PR template
   - Request reviews from maintainers

## 📝 Code Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for consistent formatting:
  ```bash
  cargo fmt --all
  ```
- Document all public APIs with `///` doc comments
- Write unit tests for new functionality

## 🐛 Reporting Issues

Found a bug? Please open an issue with:
- A clear title and description
- Steps to reproduce
- Expected vs actual behavior
- Environment details
- Relevant logs/screenshots

## 🤝 Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## 🙏 Thank You!

Your contributions make open source awesome! Thank you for helping improve Zeta Reticula.
