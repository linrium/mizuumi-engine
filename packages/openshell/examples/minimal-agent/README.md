# Minimal sandbox agent

This small POSIX shell program is an offline smoke agent. It turns a supplied
goal into a short report using only information available inside the sandbox.
The bootstrap script uploads this directory to a retained OpenShell sandbox and
executes the program there.

The output is also saved to `/sandbox/minimal-agent-result.txt` so workspace
persistence can be checked after the process exits.

