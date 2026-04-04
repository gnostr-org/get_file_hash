# Create an Issue

> `n34 issue new` Command

**Usage:**
```
Create a new repository issue

Usage: n34 issue new [OPTIONS] <--content <CONTENT>|--editor>

Options:
      --repo <NADDR-NIP05-OR-SET>  Repository address in `naddr` format (`naddr1...`), NIP-05 format (`4rs.nl/n34` or `_@4rs.nl/n34`), or a set name like `kernel`
  -c, --content <CONTENT>          Markdown content for the issue. Cannot be used together with the `--editor` flag
  -e, --editor                     Opens the user's default editor to write issue content. The first line will be used as the issue subject
      --subject <SUBJECT>          The issue subject. Cannot be used together with the `--editor` flag
  -l, --label <LABEL>              Labels for the issue. Can be specified as arguments (-l bug) or hashtags in content (#bug)
```

Use the `n34 issue new` command to create a new issue in a repository. This
command supports the [NIP-21] (`nostr:` URI scheme) and hashtags within the
issue content. When you mention public keys in the content, they will be
included in the event tags. Additionally, using hashtags like `#bug` in the
issue body will automatically apply them as labels.

You must choose between the `--content` and `--editor` options. With
`--content`, you provide the issue content directly in the command. With
`--editor`, your default `$EDITOR` will open, allowing you to write the issue
content. The first line of the editor's output will be used as the issue
subject.
