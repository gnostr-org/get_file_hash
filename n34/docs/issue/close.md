# Closes an Open Issue

> `n34 issue close` command

**Usage:**
```
Closes an open issue

Usage: n34 issue close [OPTIONS] <ISSUE_ID>

Arguments:
  <ISSUE_ID>  The open issue id to close it

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

Issue a kind `1632` (Close status) for the specified issue. The issue have to
be open.
