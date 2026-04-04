# Apply an Open Patch

> `n34 patch apply` command

**Usage:**
```
Set an open patch status to applied

Usage: n34 patch apply [OPTIONS] <PATCH_ID> [APPLIED_COMMITS]...

Arguments:
  <PATCH_ID>            The open patch id to apply it. Must be orignal root patch or revision root
  [APPLIED_COMMITS]...  The applied commits

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
      --patches <PATCH-EVENT-ID>   Patches that have been applied. Use this when only some patches have been applied, not all
```

Creates a kind `1631` event (Applied/Merged status) for the specified patch. The
patch must be in open status.

You can specify either an original patch or revision patch ID, but the status
event will only reference the original patch. Revision patches will be mentioned
in the event.

The `APPLIED_COMMITS` field serves to inform clients about the status of
specific commits, whether they have been applied or not. If you need to retrieve
the list of commits from a specific point (such as the tip of the master branch)
up to the `HEAD`, you can use the following Git command: `git log --pretty=%H
'origin/master..HEAD'`.
