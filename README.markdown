# swiftlint-autodetect

Python script to autodetect a base set of swiftlint rules.

## Installation

```sh
brew install pipx
pipx install git+https://github.com/schwa/swiftlint-autodetect.git

# to uninstall later
pipx uninstall swiftlint-autodetect
```

## Usage

Casey Liss wrote up a good description of how to use swiftlint-autodetect here: https://www.caseyliss.com/2021/12/29/swiftlint-autodetect

```sh
$ swiftlint-autodetect generate ~/Projects/MyProject
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

## Counting Violations

To show an ordered list of rules, and the number of violations per rule use the `count` subcommand.

This subcommand also highlights rules that can be corrected (marked with a green asterisk) with `swiftlint --fix`

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

## Getting minimum values for count based rules

You can also use swiftlint-autodetect to compute the minimum count of a count based rule like: line_length, file_length,
cyclomatic_complexity.

```sh
$ swiftlint-autodetect minimize ~/Projects/Demos function_body_length
function_body_length:
  error: 79
  warning: 79
only_rules:
- function_body_length
```
## How this works

swiftlint-autodetect queries swiftlint for the full list of rules and creates a temporary swiftlint config file enabling all these rules. It then performs a lint operation on the source code at the path specified and finds out which rules would be violated. It then outputs a configuration disabling the violated rules.
