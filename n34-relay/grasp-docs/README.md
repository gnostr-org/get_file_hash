# Grasp - Git Relays Authorized via Signed-Nostr Proofs

Status: DRAFT - expect breaking changes

Contributions to open-source projects shouldn't be permissioned by a platform like GitHub. Git repoistory hosting should be distributed. Grasp is a protocol like blossom, but for git.

## Overview

There may be many grasp servers anywhere -- like Blossom servers -- that host repositories from anyone (maybe they'll ask for a pre-payment, maybe they will have a free quota for some Nostr users and so on) that you can just push your repositories to. And your pushes are pre-authorized by publishing a Nostr event beforehand that says what is your repository state (branch=commit, HEAD=branch or something like that).

Then when announcing your repository you can include multiple git+http URLs to these servers that people can clone the project from. And Git-enabled Nostr clients can contact these servers to download and display source code and Git history data.

## Specification

GRASP-01 is required. Everything else is optional.

* GRASP-01 - Core Service Requirements
* GRASP-02 - Proactive Sync
* GRASP-05 - Archive

Reference implementation - [ngit-relay](https://gitworkshop.dev/npub15qydau2hjma6ngxkl2cyar74wzyjshvl65za5k5rl69264ar2exs5cyejr/ngit-relay)


TODO:
- Service Announcements and Discovery
