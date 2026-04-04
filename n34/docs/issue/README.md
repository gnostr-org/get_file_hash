# Issue Management

Using `n34`, you can manage Git issues stored in Nostr relays, adhering to
the [NIP-34] standard. In Nostr, events are immutable, meaning their IDs are
derived from the SHA-256 hash of their timestamp, content, author, and tags.
As a result, issues cannot be edited directly. However, with `n34`, you can
create new issues, view existing ones, or update their status—such as closing,
resolving, or reopening them.

[NIP-34] introduces support for drafting issues, though this feature is not
currently implemented in `n34` due to the lack of a clear use case for drafting
issues. The inclusion of this functionality may stem from its shared use in both
issues and patches, suggesting it was primarily designed for patch management.

[NIP-34]: https://github.com/nostr-protocol/nips/blob/master/34.md
