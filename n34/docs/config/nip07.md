# NIP-07 Browser Signer Proxy

> `n34 config nip07`

**Usage:**
```
Manage the NIP-07 browser signer proxy by enabling or disabling it and configuring the `ip:port` address.

Usage: n34 config nip07 [OPTIONS] <--enable|--disable>

Options:
      --enable       Enable NIP-07 as the default signer
      --disable      Disable NIP-07 as the default signer
      --addr <ADDR>  Set the `ip:port` for the browser signer proxy (default: 127.0.0.1:51034)
```

Use [NIP-07] (Browser Extension Signer) as your default signer. This is achieved
by running a proxy at the specified `ADDR`, which defaults to `127.0.0.1:51034`.
The proxy forwards `n34` requests to the browser signer and relays the responses
back.

[NIP-07]: https://github.com/nostr-protocol/nips/blob/master/07.md
