# Fetch a Patch By ID

> `n34 patch fetch` command

**Usage:**
```
Fetches a patch by its id

Usage: n34 patch fetch [OPTIONS] <PATCH_ID>

Arguments:
  <PATCH_ID>  The patch id to fetch it

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
  -o, --output <PATH>              Output directory for the patches. Default to the current directory
```

Fetches patches using their original patch ID. All fetched patches will be saved
to the specified output directory (current directory by default). You can then
apply or merge these patches into your branch as needed.
