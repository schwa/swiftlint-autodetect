import subprocess
from pathlib import Path
from shutil import which
import shlex
from typing import Dict
import yaml
import re
import sys
import typer
from collections import Counter
from typer import echo
from typer import style

app = typer.Typer()

class SwiftLint:

    def __init__(self):
        #self._rules = {}
        pass

    @property
    def rules(self):
        if hasattr(self, '_rules'):
            return self._rules
        else:
            swiftlint = which("swiftlint")
            output = (
                subprocess.check_output(shlex.split(f"{swiftlint} rules"), cwd="/tmp")
                .decode("utf8")
                .splitlines()
            )

            keys = [field.strip() for field in output[1].split("|")][1:-1]
            rows = output[3:-1]

            self._rules = {}

            for row in rows:
                values = [field.strip() for field in row.split("|")][1:-1]
                rule = dict(zip(keys, values))
                self._rules[rule['identifier']] = rule
            return self._rules

    def generateCompleteConfig(self):
        rules = list(self.rules.values())
        config = {
            "only_rules": [
                rule["identifier"] for rule in rules if rule["analyzer"] == "no"
            ],
            "analyzer_rules": [
                rule["identifier"] for rule in rules if rule["analyzer"] == "yes"
            ],
        }

        yaml.safe_dump(config, open("/tmp/swiftlint.yml", "w"))

    def lint(self, path: str):
        swiftlint = which("swiftlint")

        styles = ["lint"]
        # TODO: analyze currently broken
        # styles = ["lint", "analyze"]

        path = Path(path).expanduser()

        for style in styles:
            args = shlex.split(
                f"{swiftlint} {style} --config /tmp/swiftlint.yml --quiet {path}"
            )
            output = subprocess.run(args, capture_output=True, text=True, cwd="/tmp")
            if output.stderr:
                echo(output.stderr, err=True)
                exit(-1) # TODO: use typer errors

            # TODO: only performs first style
            return output.stdout




@app.command()
def count(path: str):
    swiftlint = SwiftLint()
    swiftlint.generateCompleteConfig()
    output = swiftlint.lint(path)
    pattern = re.compile(r"^.+:\d+:\d+: (?:warning|error): .+ \((?P<rule>.+)\)$")

    rules = Counter()

    lines = output.splitlines()

    for line in lines:
        match = pattern.match(line.strip())
        rule = match.groupdict()["rule"]
        rules[rule] += 1

    for key, value in reversed(sorted(rules.items(), key=lambda kv:kv[1])):
        rule = swiftlint.rules[key]
        correctable = style(" (*)", fg=typer.colors.GREEN) if rule["correctable"] == "yes" else ""
        echo(f"{key}{correctable}: {value}")




@app.command()
def generate(path: str):
    swiftlint = which("swiftlint")

    output = (
        subprocess.check_output(shlex.split(f"{swiftlint} rules"), cwd="/tmp")
        .decode("utf8")
        .splitlines()
    )

    keys = [field.strip() for field in output[1].split("|")][1:-1]
    rows = output[3:-1]

    rules = []

    for row in rows:
        values = [field.strip() for field in row.split("|")][1:-1]
        rule = dict(zip(keys, values))
        rules.append(rule)

    config = {
        "only_rules": [
            rule["identifier"] for rule in rules if rule["analyzer"] == "no"
        ],
        "analyzer_rules": [
            rule["identifier"] for rule in rules if rule["analyzer"] == "yes"
        ],
    }

    yaml.safe_dump(config, open("/tmp/swiftlint.yml", "w"))

    pattern = re.compile(r"^.+:\d+:\d+: (?:warning|error): .+ \((?P<rule>.+)\)$")
    failing_rules = set()

    styles = ["lint"]
    # TODO: analyze currently broken
    # styles = ["lint", "analyze"]

    path = Path(path).expanduser()

    for style in styles:
        args = shlex.split(
            f"{swiftlint} {style} --config /tmp/swiftlint.yml --quiet {path}"
        )
        output = subprocess.run(args, capture_output=True, text=True, cwd="/tmp")
        if output.stderr:
            echo(output.stderr, err=True)
            exit(-1)

        lines = output.stdout.splitlines()
        for line in lines:
            match = pattern.match(line.strip())
            rule = match.groupdict()["rule"]
            failing_rules.add(rule)

    for line in Path("/tmp/swiftlint.yml").open().readlines():
        line = line[:-1]
        for rule in failing_rules:
            if f"- {rule}" in line:
                line = line.replace(f"- {rule}", f"# - {rule}")
                break
        echo(line)
