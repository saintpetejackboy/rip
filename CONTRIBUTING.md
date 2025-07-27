# Contributing to RIP

We welcome contributions to the RIP project! To make the process smooth for everyone, please follow these guidelines.

## How to Contribute

1.  **Fork the Repository**: Start by forking the `rip` repository to your GitHub account.

2.  **Clone Your Fork**: Clone your forked repository to your local machine:
    ```bash
    git clone https://github.com/your-username/rip.git
    cd rip
    ```

3.  **Create a New Branch**: Create a new branch for your feature or bug fix. Use a descriptive name:
    ```bash
    git checkout -b feature/your-feature-name
    # or
    git checkout -b bugfix/issue-description
    ```

4.  **Make Your Changes**: Implement your feature or fix the bug. Ensure your code adheres to the existing style and conventions.

5.  **Test Your Changes**: Before submitting, make sure your changes work as expected and don't introduce regressions. Run the test suite:
    ```bash
    cargo test
    ```
    If you're adding a new feature, please write new tests for it.

6.  **Format and Lint**: Ensure your code is properly formatted and passes lint checks:
    ```bash
    cargo fmt
    cargo clippy
    ```

7.  **Commit Your Changes**: Write clear and concise commit messages. Reference any relevant issue numbers.
    ```bash
    git commit -m "feat: Add amazing new feature" # or "fix: Resolve bug #123"
    ```

8.  **Push to Your Fork**: Push your local branch to your forked repository on GitHub:
    ```bash
    git push origin feature/your-feature-name
    ```

9.  **Create a Pull Request (PR)**: Go to the original `rip` repository on GitHub and create a new Pull Request from your branch. Provide a detailed description of your changes and why they are necessary.

## Code of Conduct

Please note that this project has a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Reporting Bugs

If you find a bug, please open an issue on our [GitHub Issues page](https://github.com/saintpetejackboy/rip/issues). Provide as much detail as possible, including steps to reproduce the bug, expected behavior, and actual behavior.

## Feature Requests

We'd love to hear your ideas for new features! Please open an issue on our [GitHub Issues page](https://github.com/saintpetejackboy/rip/issues) to propose new features or improvements.
