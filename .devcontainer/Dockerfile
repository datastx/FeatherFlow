FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

USER root
# Install Node.js 18.x and necessary build dependencies
RUN apt-get update && apt-get install -y \
    wget \
    unzip \
    cmake \
    build-essential \
    git \
    ripgrep \
    ca-certificates \
    curl \
    gnupg \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && apt-get update \
    && apt-get install -y gh \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install Node.js 18.x from NodeSource repository
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get update \
    && apt-get install -y nodejs \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install DuckDB CLI
RUN wget -O /tmp/duckdb.zip https://github.com/duckdb/duckdb/releases/download/v0.9.2/duckdb_cli-linux-aarch64.zip \
    && unzip /tmp/duckdb.zip -d /usr/local/bin \
    && chmod +x /usr/local/bin/duckdb \
    && rm /tmp/duckdb.zip

# Create a directory for global npm packages
RUN mkdir -p /home/vscode/.npm-global
RUN chown -R vscode:vscode /home/vscode/.npm-global

USER vscode
# Configure npm to use the new directory path
ENV NPM_CONFIG_PREFIX=/home/vscode/.npm-global
ENV PATH=/home/vscode/.npm-global/bin:${PATH}

# Install Claude Code CLI
RUN npm install -g @anthropic-ai/claude-code