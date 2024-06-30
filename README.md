# swiftlint-autodetect

Python script to autodetect a base set of swiftlint rules.

## Installation

Via homebrew:

```sh
brew tap schwa/schwa
brew install swiftlint-autodetect
```

Or directly via cargo:

```sh
cargo install --git https://github.com/schwa/swiftlint-autodetect
```

To run `swiftlint-autodetect`, [swiftlint](https://github.com/realm/SwiftLint) must be installed and on your path.

## Usage

Casey Liss wrote up a good description of how to use swiftlint-autodetect here: <https://www.caseyliss.com/2021/12/29/swiftlint-autodetect>. This write-up is based on the previous Python version of the tool, but the usage is similar.

```sh
# Count the number of violations of _all_ SwiftLint rules in the given directory
$ swiftlint-autodetect count
# Output a SwiftLint configuration that disables all rules that are violated in the given directory.
$ swiftlint-autodetect generate --count
# Save a SwiftLint configuration that disables all rules that are violated in the given directory.
$ swiftlint-autodetect generate --count --output .swiftlint.yml
```

## Help

```plaintext
$ swiftlint-autodetect generate --help
Generate a SwiftLint configuration, by disabling rules with a minimum number of violations

Usage: swiftlint-autodetect generate [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to the project [default: .]

Options:
  -c, --counts
          Include violation counts in the generated configuration
  -o, --output <OUTPUT>
          Output path for the generated configuration
  -m, --minimum-violations <MINIMUM_VIOLATIONS>
          Minimum number of violations required to disable a rule [default: 1]
  -i, --ignore-fixable
          Don't disable fixable rules
  -h, --help
          Print help
```

## Counting Violations

To show an ordered list of rules, and the number of violations per rule use the `count` subcommand.

This subcommand also highlights rules that can be corrected with `swiftlint --fix`

```sh
$ swiftlint-autodetect count ~/Projects/Demos
explicit_acl: 2182
explicit_type_interface: 1669
identifier_name: 622
missing_docs: 409
type_contents_order: 335
explicit_top_level_acl: 321
implicit_return (*): 306
let_var_whitespace: 295
line_length: 294
force_unwrapping: 260
vertical_whitespace_between_cases (*): 167
nesting: 165
orphaned_doc_comment: 140
file_types_order: 140
indentation_width: 134
required_deinit: 117
trailing_comma (*): 106
todo: 106
```

## Configuration

Save a file at `~/.config/swiftlint-autodetect/config.toml`. Currently supported keys are 'always_disabled_rules' to specify rules that always disabled for all projects regardless of violation count, e.g.:

```toml
always_disabled_rules = ["yoda_condition"]
```

## How this works

swiftlint-autodetect queries swiftlint for the full list of rules and creates a temporary swiftlint config file enabling all these rules. It then performs a lint operation on the source code at the path specified and finds out which rules would be violated. It then outputs a configuration disabling the violated rules.
