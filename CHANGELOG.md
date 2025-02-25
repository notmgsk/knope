# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.11.0 (2023-09-13)

### Breaking Changes

#### Ignore unreachable tags when determining version

PR #574 fixes issue #505 from @BatmanAoD.

Previously, the latests tags were always used to determine the current version, **even if those tags were not reachable from `HEAD`**. Now, only reachable tags will be considered. Use the `--verbose` flag to see tags which are being ignored.

### Fixes

#### Consistent commit selection in branching histories

PR #574 fixes issue #505 from @BatmanAoD.

Previous versions of Knope did not handle branching histories correctly. In some cases, this could result in commits from previous stable releases being included in a new release. It could _also_ result in missing some commits that _should_ have been included. This has been fixed—Knope should provide you the same commit list that `git rev-list {previous_stable_tag}..HEAD` would.

## 0.10.0 (2023-09-09)

### Breaking Changes

#### Reworked Go versioning

In order to support running `Release` in a separate workflow from `PrepareRelease` and to fix a bug relating to Go module tags (when in a subdirectory), Knope will now store the full package version in a comment in the `go.mod` file and use that version as the source of truth for the package version. This has a couple of implications:

1. If you already have a comment on the `module` line in `go.mod` which matches the correct format, Knope may not be able to determine the version correctly.
2. If you have a comment on that line which does _not_ match the format, it will be erased the next time Knope bumps the version.

In either case, the solution is to erase or move that comment. Here is the syntax that Knope is looking for:

`module {ModulePath} // v{Version}`

If that comment does not exist, Knope will revert to looking for the latest relevant Git tag instead to _determine_ the version, but will still write the comment to the `go.mod` file when bumping the version.

### Features

#### `--verbose` flag

PR #545 closed issue #534 by @BatmanAoD.

There is now a global `--verbose` flag that will spit out _lots_ of extra info to stdout to assist with debugging. Right now, only the process for determining and bumping new package versions is instrumented, open issues if you need more info!

#### Allow `Release` step to be in separate workflow than `PrepareRelease`

Previously, you needed to have a `PrepareRelease` step earlier in the same workflow if you wanted to use the `Release` step. Now, if you run a `Release` step _without_ a `PrepareRelease` step, Knope will check Git tags and versioned files to figure out if there's something new to release. This is especially useful if you want to build release assets using the new version (determined by `PrepareRelease`) before actually releasing them (using `Release`).

#### Upload assets to GitHub Releases

You can now add assets to a package like this:

```toml
[package]
versioned_files = ["Cargo.toml"]
changelog = "CHANGELOG.md"

[[package.assets]]
path = "artifact/knope-x86_64-unknown-linux-musl.tgz"
name = "knope-x86_64-unknown-linux-musl.tgz"  # Optional, defaults to file name (so this `name` is doing nothing)

[[package.assets]]
path = "artifact/knope-x86_64-pc-windows-msvc.tgz"
```

When running the `Release` step with a valid `[github]` config, instead of immediately creating the release, Knope will:

1. Create a draft release
2. Upload all listed assets (erroring if any don't exist)
3. Publish the release

### Fixes

#### Use the correct tags for `go.mod` files in subdirectories

PR #544 fixed issue #502 by @BatmanAoD.

Previously, the version for a `go.mod` file was determined by the package tag, named `v{Version}` for single packages or `{PackageName}/v{Version}` for named packages. This worked when the `go.mod` file was in the root of the repository or a directory named `{PackageName}` (respectively), but not when it was in a different directory. Now, the version tag, both for determining the current version and creating a new release, will correctly be determined by the name of the directory the `go.mod` file is in (relative to the working directory). The existing package tagging strategy remains unchanged.

For example, consider this `knope.toml` file:

```toml
[package]
versioned_files = ["some_dir/go.mod"]
```

Previous to this release, creating the version `1.2.3` would only create a tag `v1.2.3`. Now, it will _additionally_ create the tag `some_dir/v1.2.3`.

## 0.9.0 (2023-08-10)

### Breaking Changes

#### Removed the deprecated `[[packages]]` syntax

If you're using the old syntax, run `knope --upgrade` _before_ switching to this version.

#### `--generate` can no longer be used if a `knope.toml` file already exists

#### Workflows can no longer be selected interactively

Previously, it was valid to invoke `knope` with no arguments, and the user would be prompted interactively to select a workflow. Now, a workflow must be provided as a positional argument, for example, `knope release`.

#### The `--prerelease-label` option can only be provided after a workflow

Previously, the `--prerelease-label` CLI option was always available globally and would simply be ignored if it was not useful for the selected workflow. Now, it can only be provided _after_ the name of a workflow which can use the option (right now, only a workflow which contains a [`PrepareRelease`](https://knope-dev.github.io/knope/config/step/PrepareRelease.html) step). For example, with the default workflow, `knope release --prerelease-label="rc"` is valid, but **none of these are valid**:

- `knope --prerelease-label="rc" release`
- `knope document-change --prerelease-label="rc"`

#### `--upgrade` can no longer be used if there is no `knope.toml` file

#### `--validate` can no longer be used if there is no `knope.toml` file

### Features

#### Added the `--override-version` option to manually set the next version

Allows you to manually determine the next version for a [`BumpVersion`] or [`PrepareRelease`] instead of using a semantic versioning rule. This option can only be provided after a workflow which contains a relevant step. This has two formats, depending on whether there is [one package](https://knope-dev.github.io/knope/config/packages.html#a-single-package-with-a-single-versioned-file) or [multiple packages](https://knope-dev.github.io/knope/config/packages.html#multiple-packages):

1. `--override-version 1.0.0` will set the version to `1.0.0` if there is only one package configured (error if multiple packages are configured).
2. `--override-version first-package=1.0.0 --override-version second-package=2.0.0` will set the version of `first-package` to `1.0.0` and `second-package` to `2.0.0` if there are multiple packages configured (error if only one package is configured).

This closes [#497](https://github.com/knope-dev/knope/issues/497).

#### `knope --help` now lists all available workflows

## 0.8.0 (2023-06-19)

### Breaking Changes

#### Changelog entries now use 4th level headers instead of bullets

In order to support more detailed changelogs via [changesets](https://knope-dev.github.io/knope/config/step/PrepareRelease.html) (like the extra text you're seeing right now!) instead of each change entry being a single bullet under the appropriate category (e.g., `### Breaking Changes` above), it will be a fourth-level header (`####`). So, where _this_ changelog entry would have currently looked like this:

```markdown
### Breaking Changes

- Changelog entries now use 4th level headers instead of bullets
```

It now looks like what you're seeing:

```markdown
### Breaking Changes

#### Changelog entries now use 4th level headers instead of bullets

... recursion omitted
```

If a change note starts with `#### ` already (like in changesets), it will be left alone.

### Features

#### Move GitHub Release headers up a level (#467, #472)

#### Added dates to version titles

There are now release dates in both changelogs and version names on GitHub. This probably won't break your releases, but you will have a different format for release notes which could be jarring. The date is in the format `YYYY-MM-DD` and will always be based on UTC time (so if you do a release late at night on the east coast of the United States, the date will be the next day).

Previously, the changelog entry title would look like this:

```markdown
## 1.0.0
```

And now it will look like this:

```markdown
## 1.0.0 (2023-06-10)
```

#### Report files to be added to git in `--dry-run`

The [`PrepareRelease`](https://knope-dev.github.io/knope/config/step/PrepareRelease.html) adds modified files to Git. Now, when running with the `--dry-run` option, it will report which files would be added to Git (for easier debugging).

> Note: The default `knope release` workflow includes this [`PrepareRelease`] step.

#### Remove duplicate version from GitHub release name

Release notes in GitHub releases used to copy the entire section of the changelog, including the version number. Because the name of the release also includes the version, you'd see the version twice, like:

```markdown
# 1.0.0

## 1.0.0

... notes here
```

Now, that second `## 1.0.0` is omitted from the body of the release.

#### Added support for changesets

Leveraging the new [changesets crate](https://github.com/knope-dev/changesets), Knope now supports [changesets](https://github.com/changesets/changesets)! In short, you can run `knope document-change` (if using default workflows) or add the new [`CreateChangeFile`] step to a workflow to generate a new Markdown file in the `.changeset` directory. You can then fill in any additional details below the generated header in the generated Markdown file. The next time the `PrepareRelease` step runs (e.g., in the default `knope release` workflow), all change files will be consumed to help generate a new version and changelog (along with any conventional commits).

For additional details, see:

- [`PrepareRelease` step](https://knope-dev.github.io/knope/config/step/PrepareRelease.html)
- [`CreateChangeFile` step](https://knope-dev.github.io/knope/config/step/CreateChangeFile.html)
- [Packages (for how you can customize changelog sections)](https://knope-dev.github.io/knope/config/packages.html)

## 0.7.4

### Features

- Allow more changelog sections via `extra_changelog_sections` config or the default `Changelog-Note` commit footer. (#450)

## 0.7.3

### Features

- Remove any potential panics (#429)

### Fixes

- Handle merge commits in history. Thanks @erichulburd! (#443)

## 0.7.2

### Fixes

- Avoid GLIBC issues by skipping gnu builds (#407)

## 0.7.1

### Fixes

- Fix building from source / `cargo install` by upgrading to `gix`. (#383)

## 0.7.0

### Breaking Changes

- Handling of pre-release versions has been reworked to handle many more edge cases, which results in more correct (but different) behavior.

### Fixes

- Check all relevant pre-release versions, not just the latest [#334, #346]. Thanks @Shadow53!

## 0.6.3

### Fixes

- determining latest versions from tags (#323)

## 0.6.2

### Features

- Allow running default workflows (those that would be created by`--generate`) with no `knope.toml` file. (#286)

## 0.6.1

### Features

- Support PEP621 in `pyproject.toml`. (#274)

## 0.6.0

### Breaking Changes

- `PrepareRelease` now `git add`s all modified files. (#267)

### Fixes

- Remote parsing in `--generate` (#268)

## 0.5.1

### Fixes

- Do not error on `--validate` or `--dry-run` when no release will be created for `PrepareRelease`. (#265)

## 0.5.0

### Breaking Changes

- `PrepareRelease` will now error if no version-altering changes are detected.

### Features

- Support limiting commits to scopes per package. (#262)
- Allow multiple defined packages, deprecate old `[[packages]]` syntax. (#257)

## 0.4.3

### Features

- `prerelease_label` can be set at runtime with `--prerelease-label` or `KNOPE_PRERELEASE_LABEL`. (#247)

## 0.4.2

### Features

- `PrepareRelease` will create a `changelog` file if it was missing. (#240)

### Fixes

- Always set a committer on tags to resolve compatibility issue with GitLab. (#236)
- Include file paths in file-related errors. (#239)

## 0.4.1

### Features

- Add support for `go.mod` in `versioned_files`. (#228)

### Fixes

- Tags were being processed backwards starting in 0.4.0 (#227)

## 0.4.0

### Breaking Changes

- Always read all commits from previous stable release tag—not from most recent tag. This tag must be in the format v<semantic_version> (e.g., v1.2.3). If your last tag does not match that format, add a new tag before running the new version of Knope.
- When creating GitHub releases, prefix the tag with `v` (e.g., `v1.2.3) as is the custom for most tools.

### Features

- Support reading commits from projects with no tags yet. (#225)
- Support pulling current version from tags. (#224)
- Allow the `Release` step to run without GitHub config—creating a tag on release. (#216)
- Support installs from cargo-binstall

### Fixes

- update rust crate git-conventional to 0.12.0 (#203)

## 0.4.0-rc.4

## 0.4.0-rc.3

## 0.4.0-rc.2

## 0.4.0-rc.1

### Breaking Changes

- When creating GitHub releases, prefix the tag with `v` (e.g., `v1.2.3) as is the custom for most tools.

### Features

- Support installs from cargo-binstall

### Fixes

- update rust crate git-conventional to 0.12.0 (#203)

## 0.4.0-rc.0

### Breaking Changes

- When creating GitHub releases, prefix the tag with `v` (e.g., `v1.2.3) as is the custom for most tools.

### Features

- Support installs from cargo-binstall

### Fixes

- update rust crate git-conventional to 0.12.0 (#203)

## 0.3.0

### Breaking Changes

- `BumpVersion` and `PrepareRelease` now require setting a `[[packages]]` field in `knope.toml`. The path to a changelog file is no longer defined with `changelog_path` in the `PrepareRelease` step. Instead, it is set as `changelog` in `[[packages]]`.

### Features

- Support multiple versioned_files in one package.
- Specify which versioned file to bump instead of picking automatically. (#182)
- Support loading GitHub credentials from `GITHUB_TOKEN` env var (#172)

### Fixes

- update rust crate thiserror to 1.0.31 (#171)

## 0.2.1-rc.0

### Features

- Support loading GitHub credentials from `GITHUB_TOKEN` env var (#172)

### Fixes

- update rust crate thiserror to 1.0.31 (#171)

## 0.2.0

### Breaking Changes

- Rename to Knope, which has much more positive associations. (#161)
- Allow switching between pre-release prefixes instead of erroring (e.g. -alpha.1 -> -beta.0)
- `BumpVersion` now takes a `label` parameter for the `Pre` rule instead of `value`.
- `UpdateProjectFromCommits` step has been renamed to `PrepareRelease`.

### Features

- Add a `--generate` option for generating a brand-new config file with a default `release` workflow. (#159)
- Add top-level `--validate` and per-workflow `--dry-run` options. (#158)
- Add a `dry-run` option to the `PrepareRelease` step. (#139, #137)
- Add a `Release` step for generating GitHub releases. (#136)
- Support pre-releases in `UpdateProjectFromCommits`. (#132)

### Fixes

- update rust crate git2 to 0.14.2 (#157)
- Bump version before adding a pre-release component.
- Stop parsing Markdown in Changelogs to avoid errors in unimplemented features. (#127)

## 0.1.5

### Fixes

- Properly handle Windows newlines in commits (#119)

## 0.1.4

### Features

- Support the BREAKING CHANGE footer with a separate breaking description.

### Fixes

- update rust crate dialoguer to 0.9.0 and console to 0.15.0 (#114)

## 0.1.3

### Features

- You can now pass the name of a workflow as an argument to bypass the selection prompt (closes #24)

### Fixes

- Commits with extra whitespace at the end were not being recorded properly

## 0.1.2

### Fixes

- Retain property order when writing changes to package.json (#75)

## 0.1.1

### Features

- Specify a path to a changelog file in UpdateProjectFromCommits (closes #27) (#71)
- Use special version bumping rules for versions that start with 0.x (closes #37) (#65)

## 0.1.0 - 2021-01-29

- Initial release
