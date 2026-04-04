# Reopen a Closed Issue

> `n34 issue reopen` command

**Usage:**
```
Reopens a closed issue

Usage: n34 issue reopen [OPTIONS] <ISSUE_ID>

Arguments:
  <ISSUE_ID>  The ID of the closed issue to reopen

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

Issue a kind `1630` (Open status) for the specified issue. The issue have to
be closed.
