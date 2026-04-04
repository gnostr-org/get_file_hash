# Secret Key Keyring

> `n34 config keyring` command

**Usage:**
```
Manages the secret key keyring, including enabling, disabling, or resetting it.

Usage: n34 config keyring <--enable|--disable|--reset>

Options:
      --enable   Enables the secret key keyring. You will be prompted for your key one last time to store it.
      --disable  Disables the secret key keyring. This removes the stored key and prevents new ones from being saved.
      --reset    Resets the keyring. This deletes the current key, allowing a new one to be stored on the next use.
```

To avoid entering your private key for every command, you can enable the keyring
to store it securely. First, run `n34 config keyring --enable`. The next time
you run an `n34` command that requires your private key, it will be saved
to your system's keyring. You will not need to enter it again for subsequent
commands.

To replace the stored key with a new one, use the `--reset` flag. To stop using
the keyring and remove the stored key, use the `--disable` flag.

`n34` uses your operating system's native secret management system. For example,
it uses `keyutils` on Linux and `Keychain` on macOS.
