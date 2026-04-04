# Command-Line Usage

## Options

The `n34` command-line tool accepts the following options:

-   `-s`, `--secret-key`: Your Nostr secret key (in `nsec` format), used for
  signing events.
-   `-b`, `--bunker-url`: The URL of a NIP-46 bunker service used for remote
  signing of events.
-   `-7`, `--nip07`: Enables signing events using the browser's NIP-07
  extension. Listens on `127.0.0.1:51034`. You can configure the address with `n34
  config nip07`
-   `-r`, `--relays`: A relay to read from and write to. This option can be
  specified multiple times to connect to several relays.
-   `--pow`: Sets the Proof of Work difficulty required when creating events.
-   `--config`: Specifies a custom path to the configuration file (Default:
  `$HOME/.config/n34/config.toml`).
-   `-v`, `--verbose...`: Increases the logging verbosity. Can be used multiple
  times for more detail (e.g., `-v`, `-vv`).

**Note:** The `--secret-key` and `--bunker-url` options are mutually exclusive.
You must provide exactly one signing method.

## Multiple Repositories

Commands that interact with a repository, such as submitting an issue or a
patch, can accept multiple repository addresses (`naddr`). This feature is
useful for projects with multiple maintainers who each have their own repository
fork.

> **Important:** When you provide multiple repositories, `n34` does not
create a separate issue or patch for each one. Instead, it creates a single
event that references all of the specified repositories.

## The `nostr-address` File

The `nostr-address` file is a plain text file that stores a list of project
repository addresses. This allows the `n34` to find and use them
without requiring you to enter the addresses manually.

### Format

- Each line must contain a single addressable event coordinate `naddr` which is
  the repository address.
- Lines beginning with a `#` are treated as comments and are ignored.
- Empty lines are also ignored.

## Passing repositories

By default, `n34` will look for a `nostr-address` file to extract repositories
from it. This is why repositories are not required for commands like `patch
send` and `issue new`. You can also pass repositories using the `--repo`
option or the `<NADDR-NIP05-OR-SET>` argument for commands that accept them. The
supported formats for manual input are:

- A [NIP-19] addressable event coordinate `naddr`.
- A [NIP-05] identifier and repository name, in the format
  `<nip05>/<repo-name>`.
- A set name that contains repository addresses.

You do not need to specify relays for these commands if your `naddr` or `NIP-05`
identifier already includes relays; `n34` will automatically extract them.

[NIP-19]: https://github.com/nostr-protocol/nips/blob/master/19.md
[NIP-05]: https://github.com/nostr-protocol/nips/blob/master/05.md
