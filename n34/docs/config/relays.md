# Fallback Relays

> `n34 config relays` command

**Usage:**
```
Sets the default fallback relays if none provided. Use this relays for read and write

Usage: n34 config relays [OPTIONS] [RELAYS]...

Arguments:
  [RELAYS]...  List of relay URLs to append to fallback relays. If empty, removes all fallback relays

Options:
      --override  Replace existing fallback relays instead of appending new ones
```

This command configures the default fallback relays, which `n34` uses to read
from and write to. To add relays, provide their URLs as arguments to append
them to the current list. Use the `--override` flag to replace the existing list
entirely. To clear all fallback relays, run the command without any arguments.
