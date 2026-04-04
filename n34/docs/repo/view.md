# View Git Repository Details

> `n34 repo view` command

**Usage:**
```
View details of a nostr git repository

Usage: n34 repo view [NADDR-NIP05-OR-SET]...

Arguments:
  [NADDR-NIP05-OR-SET]...  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

This command prints repository details to standard output. If no arguments
are provided, it looks for a `nostr-address` file in the current directory
and displays the details for the address specified within it. See [passing
repositories] for details on accepted formats.

[passing repositories]: /commands.html#passing-repositories
