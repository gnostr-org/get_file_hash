# NIP-46 Bunker

> `n34 config bunker` command

**Usage:**
```
Sets a URL of NIP-46 bunker server used for signing events

Usage: n34 config bunker [BUNKER_URL]

Arguments:
  [BUNKER_URL]  Nostr Connect URL for the bunker. Omit this to remove the current bunker URL
```

This command configures `n34` to use a remote signer ([NIP-46]), known as a
bunker, for all cryptographic operations.

When `n34` communicates with the bunker, it uses a persistent, locally-generated
keypair. You should add this keypair's public key to your bunker's list of
authorized applications. This allows `n34` to operate securely without needing
direct access to your main private key.

Once configured, actions such as fetching your public key or signing events are
delegated to the bunker. To remove the bunker configuration, run the command
again without providing a URL.

[NIP-46]: https://github.com/nostr-protocol/nips/blob/master/46.md
