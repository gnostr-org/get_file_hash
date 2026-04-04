# n34

A CLI to interact with NIP-34 and other stuff related to code in Nostr

## About

`n34` is a command-line interface (CLI) tool for sending and receiving Git
issues, patches, and comments over the Nostr protocol. It supports creating,
replying to, and managing issues and patches, making Git collaboration
decentralized and censorship-resistant.

The primary goal of `n34` is to implement [NIP-34] (`git` stuff), but its
flexible design allows for additional use cases beyond Git workflows. For more
details, see the following section.

## Documentation

Check the documentation at [n34.dev]

## Features

- [X] Repository announcements
- [ ] Repository state announcements
- [X] Patches (Send, fetch and list)
- [X] Issues (Send, view and list)
- [X] Replies
- [X] Issues and patches status
- [ ] Pull requests ([nostr-protocol/nips#1966])
- [X] Gossip Model ([NIP-65])
- [X] Proof of Work ([NIP-13])
- [X] `nostr:` URI scheme, in the issue/reply content ([NIP-21])
- [X] Signing using bunker ([NIP-46])
- [X] Signing using [NIP-07] proxy ([nostr-browser-signer-proxy])
- [ ] Code Snippets ([NIP-C0])
- [X] In device relays and repos bookmark (`sets` command)


## Why Nostr?

Nostr is fundamentally different from traditional platforms because it’s not
an application or service, it’s a decentralized protocol. This means any tool or
app can integrate with it, enabling open, permissionless collaboration without
relying on centralized gatekeepers. Unlike proprietary systems, Nostr doesn’t
require emails, passwords, or accounts. You interact directly through relays,
whether you self-host your own or use public ones, ensuring no single point of
failure or control.

What makes Nostr uniquely resilient is its design, the protocol itself is just
a set of rules, not a company or product that can disappear. Your Git issues,
patches, and comments persist as long as relays choose to store them, immune to
the whims of corporate shutdowns or policy changes. Nostr is infrastructure in
its purest form, an idea that outlives any temporary implementation. `n34` taps
into a future-proof foundation for decentralized collaboration.

### More about Nostr

- <https://nostr.com>
- <https://nostr.org>
- <https://nostr.how/en/what-is-nostr>

## Installation

You can install n34 either by cloning the repository and building it with Cargo,
or by using `cargo install` or Nix.

### Building from source

- Clone the repository:
```sh
git clone git://git.4rs.nl/awiteb/n34.git
cd n34
```

- Build the release version:
```sh
cargo build --release
```
The binary will be available at `target/release/n34`.

### Using cargo install

```sh
cargo install n34
```
The binary will be installed to your Cargo binary directory (typically `~/.cargo/bin/n34`).

Make sure `~/.cargo/bin` is in your `PATH` environment variable to run the binary from anywhere.

### Using `nix build` (+v0.4)

- Clone the repository.
- Run the `nix build` command.

The binary will be available at `result/bin/n34`.

### Adding it to your [home-manager] (+v0.4)

- Add it as an input to your `flake.nix`:

```nix
inputs = {
  # Specify the version you want to install, or remove `?ref` for the unreleased
  # version. You can also use any mirror; it doesn't have to be `git.4rs.nl`.
  n34.url = "git+https://git.4rs.nl/awiteb/n34.git?ref=refs/tags/vx.y.x";
};
```

- Add it to your packages (ensure your home-manager `extraSpecialArgs` includes the `inputs`):

```nix
packages = [ inputs.n34.packages."${pkgs.system}".default ];
```

## Contributing

Contributions to `n34` are welcome! You can help by opening issues (such as bug
reports or feature requests) or submitting patches. **All contributions must be
submitted through Nostr**. For more details on the process, please refer to the
[CONTRIBUTING.md](CONTRIBUTING.md) file. Your support is greatly appreciated!

## Contributions & Changes

You can find the changelog at [CHANGELOG.md](CHANGELOG.md) and the list of
contributors at [AUTHORS](AUTHORS) file.

## License

n34 is licensed under the GPL-3.0 License. This means that you are free to use,
modify, and distribute the software under the terms of this license. Please
refer to the [LICENSE](LICENSE) file for more details.

[NIP-34]: https://github.com/nostr-protocol/nips/blob/master/34.md
[NIP-65]: https://github.com/nostr-protocol/nips/blob/master/65.md
[NIP-13]: https://github.com/nostr-protocol/nips/blob/master/13.md
[NIP-21]: https://github.com/nostr-protocol/nips/blob/master/21.md
[NIP-C0]: https://github.com/nostr-protocol/nips/blob/master/C0.md
[NIP-46]: https://github.com/nostr-protocol/nips/blob/master/46.md
[NIP-07]: https://github.com/nostr-protocol/nips/blob/master/07.md
[nostr-protocol/nips#1966]: https://github.com/nostr-protocol/nips/pull/1966
[nostr-browser-signer-proxy]: https://crates.io/crates/nostr-browser-signer-proxy
[home-manager]: https://github.com/nix-community/home-manager
[n34.dev]: https://n34.dev
