FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

USER root
# Install build dependencies
RUN apt-get update && apt-get install -y \
    wget \
    unzip \
    cmake \
    build-essential \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

USER vscode