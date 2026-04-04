# Resolves an Issue

> `n34 issue resolve` command

**Usage:**
```
Resolves an issue

Usage: n34 issue resolve [OPTIONS] <ISSUE_ID>

Arguments:
  <ISSUE_ID>  The issue id to resolve it

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

Issue a kind `1631` (Resolved status) event for the specified issue.
