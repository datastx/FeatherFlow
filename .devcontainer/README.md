# FeatherFlow Development Container

This directory contains configuration for a Visual Studio Code Development Container that provides a consistent development environment for the FeatherFlow project.

## Features

- Rust development environment with Rust Analyzer
- DuckDB CLI for database operations
- Docker-in-Docker support for container-based workflows
- Node.js environment with NPM
- GitHub CLI for repository operations
- Claude Code CLI for AI-assisted development

## Usage

1. Install [Visual Studio Code](https://code.visualstudio.com/) and the [Remote - Containers](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) extension
2. Clone the FeatherFlow repository
3. Open the repository in VS Code
4. When prompted, click "Reopen in Container" or use the command palette (F1) and select "Remote-Containers: Reopen in Container"

## Docker Support

The container includes Docker-in-Docker support, allowing you to build and run Docker containers from within the development environment. This is useful for testing containerized applications and services without affecting your host system.

## Customization

To customize the development container:

- Modify `devcontainer.json` to add features or change settings
- Update `Dockerfile` to install additional dependencies

After making changes, rebuild the container using the "Remote-Containers: Rebuild Container" command in VS Code.