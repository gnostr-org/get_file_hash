# Reopens a Closed or Drafted Patch

> `n34 patch reopen` command

**Usage:**
```
Reopens a closed or drafted patch

Usage: n34 patch reopen [OPTIONS] <PATCH_ID>

Arguments:
  <PATCH_ID>  The closed/drafted patch id to reopen it. Must be orignal root patch

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

Issue a kind `1632` (Close status) for the specified patch. The patch have to
be closed or drafted.

