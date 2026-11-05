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
4. [Podman][Prerequisites3]
5. [A supported coding agent][Prerequisites4] that is run from within the devcontainer

[Prerequisites0]: https://herdr.dev/tui/
[Prerequisites1]: https://www.gnu.org/software/bash/ 
[Prerequisites2]: https://github.com/devcontainers/cli/
[Prerequisites3]: https://podman.io/
[Prerequisites4]: https://herdr.dev/tui/docs/agents/

## How to use it

In a Herdr session, start your devcontainer and agent in a single command like so:

```bash
dc-agent opencode
```

or

```bash
dc-agent claude
```

Replace `opencode` or `claude` with the agent TUI command name. 

If no agent name is provided, the `dc-agent` command will run whatever default agent is set in a configuration file. See the "Optional configuration" section below for more details.

## How it works

The script is a wrapper around the devcontainer CLI command used to start the devcontainer and invoke the agent session. The wrapper script sets [the environment variable][HowItWorks0] `HERDR_AGENT` with the agent name.

[HowItWorks0]: https://herdr.dev/tui/docs/agents/#vms-and-sandbox-wrappers

## Optional configuration

A default agent can be set in a configuration file. Setting one allows you to run the `dc-agent` command with no parameters. The path to the configuration file is `$HOME/.config/herdr-devcontainer-agent-opener/config.toml`. The file should consist of only one key/value pair, as follows:

```toml
default_agent = "opencode"
```