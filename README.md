# herdr-devcontainer-agent-opener

## Purpose

This repository contains a script that can be used to enter a coding agent session within a [devcontainer][Purpose0] in [herdr TUI][Purpose1] and have its agent shown in herdr's [agents pane][Purpose2].

[Purpose0]: https://containers.dev/
[Purpose1]: https://herdr.dev/tui/
[Purpose2]: https://herdr.dev/tui/docs/agent-automation/

## Prerequisites

This script relies on the following tools:

1. [Herdr TUI][Prerequisites0]
2. [Bash][Prerequisites1]
3. [Devcontainer CLI][Prerequisites2]
4. A container engine supported by the devcontainer CLI, such as [Docker][Prerequisites3] or [Podman][Prerequisites4]
5. [A supported coding agent][Prerequisites5] installed inside the devcontainer

[Prerequisites0]: https://herdr.dev/tui/
[Prerequisites1]: https://www.gnu.org/software/bash/ 
[Prerequisites2]: https://github.com/devcontainers/cli/
[Prerequisites3]: https://www.docker.com/
[Prerequisites4]: https://podman.io/
[Prerequisites5]: https://herdr.dev/tui/docs/agents/

## How to use it

In a Herdr session, start your devcontainer and agent in a single command like so:

```bash
open-agent-in-devcontainer opencode
```

or

```bash
open-agent-in-devcontainer claude
```

Replace `opencode` or `claude` with the agent TUI command name. 

If no agent name is provided, the `open-agent-in-devcontainer` command will run whatever default agent is set in a configuration file. See the "Optional configuration" section below for more details.

## How it works

The script is a wrapper around the devcontainer CLI command used to start the devcontainer and invoke the agent session. The wrapper script sets [the environment variable][HowItWorks0] `HERDR_AGENT` with the agent name.

[HowItWorks0]: https://herdr.dev/tui/docs/agents/#vms-and-sandbox-wrappers

## Optional configuration

Settings can be placed in a configuration file at `$HOME/.config/herdr-devcontainer-agent-opener/config.toml`. The file is optional, and so is every key in it. Values must be double-quoted strings.

```toml
default_agent = "opencode"
docker_path = "podman"
```

### `default_agent`

The agent TUI command name to run when `open-agent-in-devcontainer` is invoked with no agent name. Setting it allows you to run the `open-agent-in-devcontainer` command with no parameters. An agent name given on the command line always takes precedence. If no agent name is given and no `default_agent` is set, `open-agent-in-devcontainer` exits with an error.

### `docker_path`

The container CLI that the devcontainer CLI should use, passed to it as `--docker-path`. Set it if you use Podman or have a container CLI that is not `docker` on your `PATH`. The value can be a command name or a full path:

```toml
docker_path = "podman"
```

```toml
docker_path = "/usr/local/bin/podman"
```

If `docker_path` is not set, `--docker-path` is not passed and the devcontainer CLI uses its default (`docker`).