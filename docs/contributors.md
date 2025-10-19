# Contributor Guide

Thank you for your interest in contributing to the Klipper in Rust project! We welcome contributions of all kinds, from bug reports and documentation improvements to new features and code cleanups.

This document outlines the process for contributing to the project to ensure a smooth and effective workflow for everyone.

## Code of Conduct

All contributors are expected to follow our [Code of Conduct](./CODE_OF_CONDUCT.md). Please make sure you have read and understood it before proceeding. We are committed to fostering an open and welcoming environment.

## How to Contribute

There are many ways to contribute to the project:

*   **Reporting Bugs**: If you find a bug, please open an issue in our GitHub repository. Provide as much detail as possible, including:
    *   Your hardware and software versions.
    *   Steps to reproduce the bug.
    *   Expected behavior and actual behavior.
    *   Any relevant logs or error messages.
*   **Suggesting Enhancements**: If you have an idea for a new feature or an improvement to an existing one, please open an issue to discuss it. This allows us to coordinate efforts and ensure the feature aligns with the project's goals.
*   **Improving Documentation**: Good documentation is crucial. If you find something unclear, incorrect, or missing, please submit a pull request with your improvements.
*   **Writing Code**: If you want to contribute code, please follow the development workflow described below.

## Development Workflow

1.  **Fork the Repository**: Start by forking the main repository to your GitHub account.

2.  **Clone Your Fork**: Clone your forked repository to your local machine.
    ```bash
    git clone https://github.com/your-username/klipper-rust.git
    cd klipper-rust
    ```

3.  **Create a Branch**: Create a new branch for your changes. Use a descriptive name that summarizes your work.
    ```bash
    git checkout -b feature/my-new-feature
    ```

4.  **Make Your Changes**:
    *   Write your code, following the existing coding style and conventions.
    *   Add or update documentation as needed.
    *   Add unit tests for any new functionality.
    *   Ensure all existing tests pass.

5.  **Commit Your Changes**: Use clear and descriptive commit messages. We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.
    ```bash
    git commit -m "feat: Add support for TMC2209 stepper drivers"
    ```

6.  **Push to Your Fork**: Push your changes to your forked repository.
    ```bash
    git push origin feature/my-new-feature
    ```

7.  **Open a Pull Request (PR)**:
    *   Go to the original repository on GitHub and open a pull request.
    *   Provide a detailed description of your changes in the PR.
    *   Link to any relevant issues.
    *   The project maintainers will review your PR, provide feedback, and merge it when it's ready.

## Coding Style and Conventions

*   **Formatting**: We use `rustfmt` to maintain a consistent code style. Please run `cargo fmt` before committing your changes.
*   **Linting**: We use `clippy` to catch common mistakes and improve code quality. Run `cargo clippy -- -D warnings` to check for lints.
*   **Safety**: Writing `unsafe` code is strongly discouraged. If it is absolutely necessary, it must be thoroughly documented and justified, explaining why it is safe.
*   **Documentation**: All public functions, structs, and modules should have clear and concise documentation comments.

## Licensing

By contributing to this project, you agree that your contributions will be licensed under the MIT License, as described in the [LICENSE](./LICENSE) file.

Thank you for helping us build a better Klipper firmware!
