# Merge an Open Patch

> `n34 patch merge` command

**Usage:**
```
Set an open patch status to merged

Usage: n34 patch merge [OPTIONS] <PATCH_ID> <MERGE_COMMIT>

Arguments:
  <PATCH_ID>      The open patch id to merge it. Must be orignal root patch or revision root
  <MERGE_COMMIT>  The merge commit id

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
      --patches <PATCH-EVENT-ID>   Patches that have been merged. Use this when only some patches have been merged, not all
```

Creates a kind `1631` event (Applied/Merged status) for the specified patch. The
patch must be in open status.

You can specify either an original patch or revision patch ID, but the status
event will only reference the original patch. Revision patches will be mentioned
in the event.

You can get the `MERGE_COMMIT` commit using `git rev-parse HEAD` command if
the merge commit in the `HEAD` or use `HEAD~n` where the `n` is the number of
commits the merge commit before the HEAD
