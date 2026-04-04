# This justfile is for the contrbutors of this project, not for the end user.
#
# Requirements for this justfile:
# - Linux distribution
# - just (Of course) <https://github.com/casey/just>
# - cargo (For the build and tests) <https://doc.rust-lang.org/cargo/getting-started/installation.html>
# - mdbook (<https://rust-lang.github.io/mdBook>)
# - git-cliff (<https://git-cliff.org>)
# - taplo (<https://taplo.tamasfe.dev/>)
# - cargo-msrv (<https://github.com/foresterre/cargo-msrv>)
# - nushell (<https://nushell.sh>)

set quiet
set unstable
set shell := ["/usr/bin/env", "bash", "-c"]
set script-interpreter := ["/usr/bin/env", "nu"]

JUST_EXECUTABLE := "just -u -f " + justfile()
header := "Available tasks:\n"
BOOK_DEST_DIR := "dest"
tag_change_body := '''{% for group, commits in commits | group_by(attribute="group") %}

{{ group | upper_first }}

{% for commit in commits %}
- {{ commit.message | split(pat="\n") | first | split(pat=":") | slice(start=1) | join(sep=":") | upper_first | trim }} - by {{ commit.author.name}}{% endfor %}{% endfor %}
'''

export TZ := "UTC"

_default:
    @{{JUST_EXECUTABLE}} --list-heading "{{header}}" --list

# Run the CI
ci: && msrv _done_ci
    echo "🔨 Building n34..."
    cargo build -q
    echo "🔍 Checking code formatting..."
    cargo fmt -q -- --check
    RUST_LOG=none taplo fmt --check --config "./.taplo.toml" || (echo "❌Toml files is not properly formatted" && exit 1)
    echo "🧹 Running linter checks..."
    cargo clippy -q -- -D warnings
    echo "🧪 Running tests..."
    cargo test -q

# Check that the current MSRV is correct
msrv:
    echo "🔧 Verifying MSRV..."
    cargo-msrv verify
    echo "✅ MSRV verification passed"

_done_ci:
    echo "🎉 CI pipeline completed successfully"

# Update the changelog
[script]
changelog:
    def get_hash [] { open "./CHANGELOG.md" | hash sha256 }

    let old_hash = get_hash
    git-cliff out> "CHANGELOG.md"

    if old_hash != get_hash {
        git add "CHANGELOG.md"
        git commit -m "chore(changelog): Update the changelog"
        print "The changes have been added to the changelog file and committed"
    } else {
        print "No changes have been added to the changelog"
    }

# Releases a new version of n34. Requires a clean file tree with no uncommitted changes.
[script]
release version:
    let tag_msg = "Version {{ version }}" + (git-cliff --strip all --unreleased --body '{{ tag_change_body }}')
    mut cargo_file = open "Cargo.toml"

    $cargo_file.package.version = "{{ version }}"
    $cargo_file | save -f "Cargo.toml"

    RUST_LOG=none taplo fmt --config "./.taplo.toml"
    {{ JUST_EXECUTABLE }} ci
    git-cliff -t "v{{ version }}" out> "./CHANGELOG.md"
    git add .
    git commit -m "chore: Bump the version to `v{{ version }}`"
    git tag -s -m $tag_msg "v{{ version }}"
    git push origin master --tags
    cargo publish

# Deploy the book to Github Pages
deploy:
    mdbook build --dest-dir {{ BOOK_DEST_DIR }}
    cd {{ BOOK_DEST_DIR }}
    git init .
    git checkout -B gh-pages
    touch ".nojekyll"
    echo "n34.dev" > "CNAME"

    git add .
    git commit -m "Deploy the book to github pages"
    git remote add origin "git@github.com:TheAwiteb/n34-book"
    git push origin gh-pages -f
    cd ..
    rm -fr {{ BOOK_DEST_DIR }}

