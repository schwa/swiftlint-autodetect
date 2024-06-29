set shell := ["fish", "-c"]

_default: _list

_list:
    just --list

publish:
    #!/usr/bin/env fish
    set CURRENT_BRANCH (git symbolic-ref --short HEAD)
    if [ $CURRENT_BRANCH != main ]
        echo "Not on main branch. Please switch to main before publishing."
        exit 1
    end
    set NEXT_VERSION (just _next-version)
    gum confirm "Confirm next version: '$NEXT_VERSION'?"; or exit 1
    just _check-repo; or exit 1
    cargo clippy --fix
    just _check-repo; or exit 1
    cargo test; or exit 1
    echo "Updating Cargo.toml to version $NEXT_VERSION"
    toml set Cargo.toml package.version $NEXT_VERSION| sponge Cargo.toml
    gum confirm "git commit -a"; and git commit -a
    gum confirm "git tag?"; and git tag $NEXT_VERSION
    gum confirm "git push?"; and git push --tags origin main
    gum confirm "Rust publish"; and cargo publish

    gum confirm "Update homebrew?"; and just homebrew-release $NEXT_VERSION

_check-repo:
    #!/usr/bin/env fish
    set is_dirty (git status --porcelain)
    if test -n "$is_dirty"
        echo "Repo is dirty. Please commit all changes before publishing."
        exit 1
    end

_next-version:
    #!/usr/bin/env fish
    set LATEST_TAG (git describe --tags --abbrev=0)
    set PARTS (string split . $LATEST_TAG)
    set MAJOR $PARTS[1]
    set MINOR $PARTS[2]
    set PATCH $PARTS[3]
    set NEXT_PATCH (math $PATCH + 1)
    echo "$MAJOR.$MINOR.$NEXT_PATCH"

homebrew-release VERSION:
    #!/usr/bin/env fish
    # https://federicoterzi.com/blog/how-to-publish-your-rust-project-on-homebrew/

    set PROJECT_NAME "swiftlint-autodetect"
    set VERSION {{VERSION}}
    echo $VERSION

    # Build release, create tarball and calculate sha256
    cargo build --release
    pushd target/release/
    tar -czf $PROJECT_NAME.tar.gz $PROJECT_NAME
    set SHA (shasum -a 256 $PROJECT_NAME.tar.gz | cut -d " " -f 1)
    echo $SHA
    popd

    # Create release on GitHub
    gh release create $VERSION target/release/$PROJECT_NAME.tar.gz --title "$PROJECT_NAME $VERSION"

    # Update homebrew formula
    pushd $HOME/Projects/homebrew-schwa
    git pull
    sed -i '' -e "s/sha256 \".*\"/sha256 \"$SHA\"/g" Formula/$PROJECT_NAME.rb
    sed -i '' -e "s/version \".*\"/version \"$VERSION\"/g" Formula/$PROJECT_NAME.rb
    git commit --all --message "$PROJECT_NAME $VERSION"
    git push
    popd

test-homebrew:
    brew tap schwa/schwa
    brew update
    brew install swiftlint-autodetect
