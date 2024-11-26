# UID-GID shifter

A small Rust command line utility to shift user and group IDs of files and folders.
This is useful for handling directories of unprivileged container environments as created by
[LXC](https://linuxcontainers.org/) or [Podman](https://podman.io/).

#### Available command line options:

* `--no-recursive`: Disables recursion into sub-directories
* `-v`, `--verbose`: Enable verbose command output)
* `--dry-run`: Skips changing the user and group IDs for trial runs
* `-u`, `--uid-offset`: The user ID offset to be applied to each file. Can both be positive or negative
* `-g`, `--gid-offset`: The group ID offset to be applied to each file. Can both be positive or negative
* `-h`, `--help`: Prints the utilities help
