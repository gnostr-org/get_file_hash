# List Repositories Issues

> `n34 issue list` command

**Usage:**
```
List the repositories issues

Usage: n34 issue list [OPTIONS] [NADDR-NIP05-OR-SET]...

Arguments:
  [NADDR-NIP05-OR-SET]...  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`

Options:
      --limit <LIMIT>  Maximum number of issues to list [default: 15]
```

List the repositories issues. By default `n34` will look for `nostr-address`
file and extract the repositories from it.
