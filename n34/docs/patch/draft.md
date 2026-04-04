# Draft an Open Patch

> `n34 patch draft` command

**Usage:**
```
Converts an open patch to draft state

Usage: n34 patch draft [OPTIONS] <PATCH_ID>

Arguments:
  <PATCH_ID>  The open patch id to draft it. Must be orignal root patch

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
```

Issue a kind `1633` (Draft status) for the specified patch. The patch have to
be open.
