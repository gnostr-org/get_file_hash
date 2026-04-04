# Default PoW Difficulty

> `n34 config pow`

**Usage:**
```
Sets the default PoW difficulty (0 if not specified)

Usage: n34 config pow <DIFFICULTY>

Arguments:
  <DIFFICULTY>  The new default PoW difficulty
```

This command configures the default Proof of Work (PoW) difficulty for newly
created events. This setting is applied to most generated events, but it
intentionally skips patch events. Because patches can be numerous, calculating
PoW for each one would significantly slow down operations.

If you want to disable the PoW just make it 0.
