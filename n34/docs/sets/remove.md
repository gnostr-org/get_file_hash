# Remove a Set

> `n34 sets remove` command

**Usage:**
```
Remove a set, or specific repos and relays within it

Usage: n34 sets remove [OPTIONS] <NAME>

Arguments:
  <NAME>  Set name to delete

Options:
      --set-relay <RELAYS>         Specific relay to remove it from the set, either as URL or set name to extract its relays. [aliases: `--sr`]
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

Removes an entire set, or specific repositories and relays from it.
Without options, this command deletes the entire set.

See the [passing repositories] section for more details on supported formats.

[passing repositories]: /commands.html#passing-repositories
