# Reply to Issues and Patches

> `n34 reply` command

**Usage:**
```
Reply to issues and patches

Usage: n34 reply [OPTIONS] <--comment <COMMENT>|--editor> <nevent1-or-note1>

Arguments:
  <nevent1-or-note1>  The issue, patch, or comment to reply to

Options:
      --quote-to                   Quote the replied-to event in the editor
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
  -c, --comment <COMMENT>          The comment (cannot be used with --editor)
  -e, --editor                     Open editor to write comment (cannot be used with --content)
```

Craft replies ([NIP-22] Comment) to issues, patches, or comments with ease
using the `n34 reply` command. You can either input your reply directly with
the `--comment` option or open an editor for a more detailed response using
`--editor`. Additionally, when using `--editor`, the `--quote-to` option
allows you to include the original content in your editor, enabling precise and
context-aware replies.

[NIP-22]: https://github.com/nostr-protocol/nips/blob/master/22.md
