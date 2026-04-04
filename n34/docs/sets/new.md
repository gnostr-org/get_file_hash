# Create a Set

> `n34 sets new` command

**Usage:**
```
Create a new set

Usage: n34 sets new [OPTIONS] <NAME>

Arguments:
  <NAME>  Unique name for the set

Options:
      --set-relay <RELAYS>         Optional relay to add it to the set, either as URL or set name to extract its relays. [aliases: `--sr`]
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

Each set requires a unique name, provided as the final argument to the command.
Use the `--set-relays`/`--sr` option to specify the relays for the new set;
this can be a relay URL or the name of an existing set whose relays you wish to
use. To add repositories, use the `--repo` option. Check [passing repositories]
format.

[passing repositories]: /commands.html#passing-repositories
