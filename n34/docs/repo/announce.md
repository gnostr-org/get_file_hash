# Broadcast and Update a Git Repository

> `n34 repo announce` command

**Usage:**
```
Broadcast and update a git repository

Usage: n34 repo announce [OPTIONS] --id <REPO_ID>

Options:
      --id <REPO_ID>               Unique identifier for the repository in kebab-case
  -n, --name <NAME>                A name for the repository
  -d, --description <DESCRIPTION>  A description for the repository
  -w, --web <WEB>                  Webpage URLs for the repository (if provided by the git server)
  -c, --clone <CLONE>              URLs for cloning the repository
  -m, --maintainers <MAINTAINERS>  Additional maintainers of the repository (besides yourself)
  -l, --label <LABEL>              Labels to categorize the repository. Can be specified multiple times
      --force-id                   Skip kebab-case validation for the repository ID
      --address-file               If set, creates a `nostr-address` file to enable automatic address discovery by n34
```

This command generates an announcement event to publish your project. It can be
used to announce a new repository or update an existing one.

When updating, you must resubmit all repository fields, not just the fields
you wish to change. The command uses this information to build and publish a
completely new announcement event that will replace the old one.

It is recommended to use the `--address-file` flag. This option creates
a `nostr-address` file that enables `n34` to automatically discover the
repository's address, simplifying the workflow for contributors.
