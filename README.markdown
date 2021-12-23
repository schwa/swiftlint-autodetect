# swiftlint-autodetect

Python script to autodetect a base set of swiftlint rules.

## Installation

```sh
brew install pipx
pipx install https://github.com/schwa/swiftlint-autodetect.git
```

## Usage

```sh
swiftlint-autodetect ~/Projects/MyProject
```

And this outputs:

```yaml
analyzer_rules:
- capture_variable
- explicit_self
- unused_declaration
- unused_import
only_rules:
# - anonymous_argument_in_multiline_closure
- anyobject_protocol
# - array_init

# and so on
```

## How this works

swiftlint-autodetect queries swiftlint for the full list of rules and creates a temporary swiftlint config file enabling all these rules. It then performs a lint operation on the source code at the path specified and finds out which rules would are violated. It then outputs a configuration disabling the violated rules.
