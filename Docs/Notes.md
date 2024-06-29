# Notes

## swiftlint help

```plaintext
$ swiftlint lint --help
OVERVIEW: Print lint warnings and errors

USAGE: swiftlint lint [<options>] [<paths> ...]

ARGUMENTS:
  <paths>                 List of paths to the files or directories to lint.

OPTIONS:
  --config <config>       The path to one or more SwiftLint configuration files, evaluated as a
                          parent-child hierarchy.
  --fix, --autocorrect    Correct violations whenever possible.
  --format                Should reformat the Swift files using the same mechanism used by Xcode (via
                          SourceKit).
                          Only applied with `--fix`/`--autocorrect`.
  --use-alternative-excluding
                          Use an alternative algorithm to exclude paths for `excluded`, which may be
                          faster in some cases.
  --use-script-input-files
                          Read SCRIPT_INPUT_FILE* environment variables as files.
  --strict                Upgrades warnings to serious violations (errors).
  --lenient               Downgrades serious violations to warnings, warning threshold is disabled.
  --force-exclude         Exclude files in config `excluded` even if their paths are explicitly
                          specified.
  --benchmark             Save benchmarks to `benchmark_files.txt` and `benchmark_rules.txt`.
  --reporter <reporter>   The reporter used to log errors and warnings.
  --baseline <baseline>   The path to a baseline file, which will be used to filter out detected
                          violations.
  --write-baseline <write-baseline>
                          The path to save detected violations to as a new baseline.
  --in-process-sourcekit  Use the in-process version of SourceKit.
  --output <output>       The file where violations should be saved. Prints to stdout by default.
  --progress              Show a live-updating progress bar instead of each file being processed.
  --use-stdin             Lint standard input.
  --quiet                 Don't print status logs like 'Linting <file>' & 'Done linting'.
  --silence-deprecation-warnings
                          Don't print deprecation warnings.
  --cache-path <cache-path>
                          The directory of the cache used when linting.
  --no-cache              Ignore cache when linting.
  --enable-all-rules      Run all rules, even opt-in and disabled ones, ignoring `only_rules`.
  --version               Show the version.
  -h, --help              Show help information.
  ```

## Swiftlint Configuration

```plaintext
# By default, SwiftLint uses a set of sensible default rules you can adjust:
disabled_rules: # rule identifiers turned on by default to exclude from running
  - colon
  - comma
  - control_statement
opt_in_rules: # some rules are turned off by default, so you need to opt-in
  - empty_count # find all the available rules by running: `swiftlint rules`

# Alternatively, specify all rules explicitly by uncommenting this option:
# only_rules: # delete `disabled_rules` & `opt_in_rules` if using this
#   - empty_parameters
#   - vertical_whitespace

analyzer_rules: # rules run by `swiftlint analyze`
  - explicit_self

# Case-sensitive paths to include during linting. Directory paths supplied on the
# command line will be ignored.
included:
  - Sources
excluded: # case-sensitive paths to ignore during linting. Takes precedence over `included`
  - Carthage
  - Pods
  - Sources/ExcludedFolder
  - Sources/ExcludedFile.swift
  - Sources/*/ExcludedFile.swift # exclude files with a wildcard

# If true, SwiftLint will not fail if no lintable files are found.
allow_zero_lintable_files: false

# If true, SwiftLint will treat all warnings as errors.
strict: false

# The path to a baseline file, which will be used to filter out detected violations.
baseline: Baseline.json

# The path to save detected violations to as a new baseline.
write_baseline: Baseline.json

# configurable rules can be customized from this configuration file
# binary rules can set their severity level
force_cast: warning # implicitly
force_try:
  severity: warning # explicitly
# rules that have both warning and error levels, can set just the warning level
# implicitly
line_length: 110
# they can set both implicitly with an array
type_body_length:
  - 300 # warning
  - 400 # error
# or they can set both explicitly
file_length:
  warning: 500
  error: 1200
# naming rules can set warnings/errors for min_length and max_length
# additionally they can set excluded names
type_name:
  min_length: 4 # only warning
  max_length: # warning and error
    warning: 40
    error: 50
  excluded: iPhone # excluded via string
  allowed_symbols: ["_"] # these are allowed in type names
identifier_name:
  min_length: # only min_length
    error: 4 # only error
  excluded: # excluded via string array
    - id
    - URL
    - GlobalAPIKey
reporter: "xcode" # reporter type (xcode, json, csv, checkstyle, codeclimate, junit, html, emoji, sonarqube, markdown, github-actions-logging, summary)
```
