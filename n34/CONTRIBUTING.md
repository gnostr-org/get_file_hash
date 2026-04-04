# Contributing to `n34`

For basic information about the `n34` project, please read the
[README.md](README.md). The project is licensed under **GPL-3.0**, and by
contributing, your work will also be licensed under the same terms.

Before submitting changes, please read the [Developer Certificate of Origin](DCO).
All patches must include a `Signed-off-by: NAME <EMAIL>` line to acknowledge
your agreement with the DCO.

Ensure your Git name and email are correctly configured. While you don’t need to
use your real details, avoid leaving them as the default values. To verify your
current settings, run:

```bash
git config user.name
git config user.email
```

If they’re incorrect or unset, update them using:

```bash
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

We welcome all contributions, whether it be bug reports, fixes, feature
submissions, feature requests, or improving documentation or testing. Enjoy
collaborating!

## Git Repository

The repository is hosted at <https://git.4rs.nl/awiteb/n34.git>, with `master`
as the active development branch.

## Nostr Repository Address

You can submit issues and patches via any
Nostr-compatible client using the address:
`naddr1qqpkuve5qgsqqqqqq9g9uljgjfcyd6dm4fegk8em2yfz0c3qp3tc6mntkrrhawgrqsqqqaueq
yf8wumn8ghj7mn0wd68yt35wfejumnvqyxhwumn8ghj7mn0wvhxcmmvqy28wumn8ghj7mn0wd68ytn00
p68ytnyv4mqwuj6xc`


When using `n34`, there's no need to specify the address, it will automatically
check the `nostr-address` file. Simply submit your issues and patches without
worrying about this detail.

## Contribution Workflow

Before submitting changes, open an issue to discuss your proposed contribution.
Clearly indicate that you intend to work on it and wait for a maintainer's
response. If the issue remains open, you may proceed with submitting your patch.

### Reporting Issues

When opening an issue, include:
- Detailed steps to reproduce the problem
- Relevant error messages or logs (use the `-vvvv` flag for verbose output)
- Expected vs. actual behavior

Please label your issue appropriately (e.g., `bug`, `feature`, `question`) to
help categorize it.

### Your Patch

Ensure your patch submission tool notifies the maintainers and sends the patch
to their read relays, most tools handle this automatically.

#### Patch Guidelines

- Keep patches small: Focused changes are easier to review and merge.
- Run `just ci` before submitting your patch.
- Update the change log with your patch. Run `just changelog` or `git-cliff > CHANGELOG.md`
- Add your name to the [AUTHORS](AUTHORS) file if this is your first contribution. (alphabetical order)
- Use [Conventional Commits]: Start the patch subject with one of these types:
  - `feat`: New feature
  - `fix`: Bug fix
  - `docs`: Documentation updates
  - `refactor`: Code restructuring without behavioral changes
  - `deprecate`: Marking code as deprecated
  - `remove`: Removing deprecated code
  - `security`: Security-related changes
  - `perf`: Performance improvements
  - `test`: Test additions or corrections
- For all other changes, use `chore`.
- Add `!` to the subject if your patch contains a breacking change, e.g.
`remove!: text` and `fix(reply)!: text`
- Use the `--cover-letter` flag to include a cover letter with your patch. Describe the issue you’re addressing, whether it’s a one-line bug fix or a 5000-line new feature.
- Specify the base commit for your patch using the `--base` flag.
- First-time contributors: Review the [Submitting Patches guide](https://www.kernel.org/doc/html/latest/process/submitting-patches.html) before sending your patch.
- If you revise your patch, you should reference all previous revisions (or the
root patch if this is the first revision) and explain the changes made (i.e.,
the differences between this patch and the prior one).

#### Code Style

When writing code, make sure to folow this:
- Using Rust's official formatting tool, `rustfmt`, to format your code.
- Writing clear and concise code with meaningful variable and function names.
- Adding comments to explain complex logic or algorithms.

#### Cover Letter Description

Your patch description should provide a clear and concise summary of the changes you
have made. It should also include any relevant context or background information
that will help the project maintainers understand the purpose of the changes.
Make sure to reference the issue[^1] that your patch is addressing, and note any breaking
changes that your patch introduces.



[^1]: When referencing, avoid URLs or `nevent` formats with relays. Instead, use only the note ID in `note1` bech32 format.

[Conventional Commits]: https://www.conventionalcommits.org/en/v1.0.0/
