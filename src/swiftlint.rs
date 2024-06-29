use anyhow::Result;
use colored_markup::{println_markup, StyleSheet};
use hashlink::LinkedHashMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use shellexpand;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use yaml_rust2::{yaml, Yaml, YamlEmitter, YamlLoader};

#[derive(Debug, Deserialize)]
struct Config {
    always_disabled_rules: Option<Vec<String>>,
}

impl Config {
    fn new() -> Self {
        Config {
            always_disabled_rules: None,
        }
    }

    fn load() -> Result<Self> {
        let path = "~/.config/swiftlint-autodetect/config.toml";
        let path = shellexpand::tilde(path).into_owned();
        // if path exists
        if Path::new(&path).exists() {
            let contents = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&contents)?;
            return Ok(config);
        }
        Ok(Config::new())
    }
}

#[derive(Debug)]
pub struct Swiftlint {
    pub binary_path: PathBuf,
    pub working_directory: PathBuf,
    pub rules: Vec<Rule>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Rule {
    pub identifier: String,
    pub opt_in: bool,
    pub correctable: bool,
    pub kind: String,
    pub analyzer: bool,
    pub uses_sourcekit: bool,
}

#[derive(Debug, Serialize)]
pub struct SwiftLintConfig {
    pub disabled_rules: Vec<String>,
    pub excluded: Vec<String>,
    pub opt_in_rules: Vec<String>,
    pub only_rules: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub character: u32,
    pub severity: String, // TODO: Make into an enu,
    pub reason: String,
    pub rule_id: String,
}

impl From<&str> for Diagnostic {
    fn from(line: &str) -> Self {
        // TODO: use --reporter=json instead.
        let pattern = Regex::new(r"^(?<path>.+):(?<line>\d+):(?<column>\d+): (?<level>warning|error): (?<description>.+) \((?<identifier>.+)\)$").unwrap();
        let captures = pattern.captures(line).unwrap();
        let path = PathBuf::from(captures.name("path").unwrap().as_str());
        let line = captures
            .name("line")
            .unwrap()
            .as_str()
            .parse::<u32>()
            .unwrap();
        let column = captures
            .name("column")
            .unwrap()
            .as_str()
            .parse::<u32>()
            .unwrap();
        let level = captures.name("level").unwrap().as_str().to_string();
        let message = captures.name("description").unwrap().as_str().to_string();
        let identifier = captures.name("identifier").unwrap().as_str().to_string();
        Diagnostic {
            file: path,
            line,
            character: column,
            severity: level,
            reason: message,
            rule_id: identifier,
        }
    }
}

impl Swiftlint {
    pub fn new(working_directory: PathBuf) -> Result<Self> {
        let output = Command::new("/usr/bin/which")
            .arg("swiftlint")
            .output()
            .expect("failed to execute process");

        let binary_path = String::from_utf8(output.stdout)?.trim().to_string();
        let binary_path = PathBuf::from(binary_path);

        let mut swift_lint = Swiftlint {
            binary_path,
            working_directory,
            rules: Vec::new(),
        };
        swift_lint.discover_rules()?;
        Ok(swift_lint)
    }

    pub fn discover_rules(&mut self) -> Result<()> {
        let stdout = Command::new(&self.binary_path)
            .arg("rules")
            .current_dir(&self.working_directory)
            .output()?
            .stdout;

        let stdout = String::from_utf8(stdout)?;
        let lines = stdout.lines();

        // Skip first line
        let mut lines = lines.skip(1);
        // Parse header
        let header = lines.next().unwrap();
        let header_parts: Vec<&str> = header.split('|').map(|part| part.trim()).collect();

        let index_for_identifier = header_parts
            .iter()
            .position(|&part| part == "identifier")
            .unwrap();
        let index_for_opt_in = header_parts
            .iter()
            .position(|&part| part == "opt-in")
            .unwrap();
        let index_for_correctable = header_parts
            .iter()
            .position(|&part| part == "correctable")
            .unwrap();
        let index_for_kind = header_parts
            .iter()
            .position(|&part| part == "kind")
            .unwrap();
        let index_for_analyzer = header_parts
            .iter()
            .position(|&part| part == "analyzer")
            .unwrap();
        let index_for_uses_sourcekit = header_parts
            .iter()
            .position(|&part| part == "uses sourcekit")
            .unwrap();

        // Skip next line
        let _ = lines.next().unwrap();

        // Trim off first three lines & last line
        let lines = lines.take_while(|line| *line != "+------------------------------------------+--------+-------------+------------------------+-------------+----------+----------------+---------------+");
        let rules: Vec<Rule> = lines
            .map(|line| {
                let parts: Vec<&str> = line.split('|').map(|part| part.trim()).collect();
                Rule {
                    identifier: parts[index_for_identifier].to_string(),
                    opt_in: parts[index_for_opt_in] == "yes",
                    correctable: parts[index_for_correctable] == "yes",
                    kind: parts[index_for_kind].to_string(),
                    analyzer: parts[index_for_analyzer] == "yes",
                    uses_sourcekit: parts[index_for_uses_sourcekit] == "yes",
                }
            })
            .collect();
        self.rules = rules;
        Ok(())
    }

    pub fn generate_config(&self) -> Result<PathBuf> {
        let dotbuild = fs::canonicalize(&self.working_directory)?.join(".build/*");

        let exclusions: Vec<&str> = vec![dotbuild.to_str().unwrap()];

        // get all non-analyzer rules
        let rules = self
            .rules
            .iter()
            .filter(|rule| !rule.analyzer)
            .collect::<Vec<&Rule>>();

        let config = SwiftLintConfig {
            disabled_rules: vec![],
            excluded: exclusions
                .iter()
                .map(|exclusion| exclusion.to_string())
                .collect(),
            opt_in_rules: rules.iter().map(|rule| rule.identifier.clone()).collect(),
            only_rules: vec![],
        };

        let yaml = serde_yml::to_string(&config)?;

        let path = PathBuf::from("/tmp/swiftlint.yml");
        let mut file = std::fs::File::create(&path)?;

        file.write_all(yaml.as_bytes())?;
        Ok(path)
    }

    pub fn lint(&self, config_path: &PathBuf) -> Result<Vec<Diagnostic>> {
        let output = Command::new(&self.binary_path)
            .args(["lint", "--quiet", "--config", config_path.to_str().unwrap()])
            .current_dir(&self.working_directory)
            .output()?;
        let stderr = String::from_utf8(output.stderr).unwrap();
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }

        let stdout = String::from_utf8(output.stdout).unwrap();

        // write stdout to /tmp/swiftlint.log
        let mut file = fs::File::create("/tmp/swiftlint.log")?;
        file.write_all(stdout.as_bytes())?;

        let diagnostics: Vec<Diagnostic> = stdout.lines().map(Diagnostic::from).collect();

        Ok(diagnostics)
    }

    pub fn count(&self) -> Result<()> {
        let style_sheet = StyleSheet::parse(
            "
        fixable { foreground: bright-green; styles: underline }
        warning { foreground: bright-yellow }
        bad { foreground: bright-red }
        ",
        )
        .unwrap();

        let path = self.generate_config().unwrap();

        let diagnostics = self.lint(&path).unwrap();

        // Count the diagnostics by identifier
        let mut diagnostics_by_identifier: HashMap<String, u32> = HashMap::new();
        for diagnostic in diagnostics.iter() {
            let count = diagnostics_by_identifier
                .entry(diagnostic.rule_id.clone())
                .or_insert(0);
            *count += 1;
        }

        // sort by most diagnostics
        let mut diagnostics_by_identifier: Vec<(String, u32)> = diagnostics_by_identifier
            .iter()
            .map(|(identifier, count)| (identifier.clone(), *count))
            .collect();

        diagnostics_by_identifier.sort_by(|a, b| b.1.cmp(&a.1));

        for (identifier, count) in diagnostics_by_identifier.iter() {
            let mut line = String::new();
            let rule = self
                .rules
                .iter()
                .find(|rule| rule.identifier == *identifier)
                .unwrap();

            if *count >= 10 {
                line.push_str(format!("{}: <bad>{}</bad>", rule.identifier, count).as_str());
            } else {
                line.push_str(
                    format!("{}: <warning>{}</warning>", rule.identifier, count).as_str(),
                );
            }
            if rule.correctable {
                line.push_str(" <fixable>fixable</fixable>");
            }
            line.push_str(format!(" ({})", rule.kind).as_str());

            println_markup!(&style_sheet, "{}", line);
        }

        Ok(())
    }

    pub fn generate(
        &self,
        output_path: Option<PathBuf>,
        include_counts: bool,
        minimum_violations: u32,
        ignore_fixable: bool,
    ) -> Result<()> {
        let app_config = Config::load()?;

        let path = self.generate_config()?;

        let diagnostics = self.lint(&path)?;
        let mut diagnostics_by_identifier: HashMap<String, u32> = HashMap::new();
        for diagnostic in diagnostics.iter() {
            let count = diagnostics_by_identifier
                .entry(diagnostic.rule_id.clone())
                .or_insert(0);
            *count += 1;
        }

        // Create a string buf to write to
        let mut output = String::new();

        if let Some(output_path) = &output_path {
            // if output path exists
            if output_path.exists() {
                // modify the yaml
                let modified_yaml = modify_yaml(output_path, vec!["disabled_rules", "only_rules"])?;
                output.push_str(&modified_yaml);
                output.push('\n');
            }
        }

        output.push_str("only_rules:\n");
        for rule in self.rules.iter() {
            let count = diagnostics_by_identifier
                .get(&rule.identifier)
                .unwrap_or(&0);
            let mut line = format!("  - {}", rule.identifier);
            if include_counts && *count != 0 {
                line = format!("{} # {} violations", line, count);
                if rule.correctable {
                    line = format!("{} (fixable)", line);
                }
            }
            let mut disabled = false;
            if let Some(always_disabled_rules) = &app_config.always_disabled_rules {
                if always_disabled_rules.contains(&rule.identifier) {
                    disabled = true;
                }
            }
            if *count >= minimum_violations && !(ignore_fixable && rule.correctable) {
                disabled = true
            }

            if disabled {
                line = format!("#{}", line);
            }

            output.push_str(format!("{}\n", &line).as_str());
        }

        if let Some(output_path) = &output_path {
            let mut file = fs::File::create(output_path)?;
            file.write_all(output.as_bytes())?;
        } else {
            // write to stdout
            println!("{}", output);
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub fn modify_yaml(path: &Path, keys_to_strip: Vec<&str>) -> Result<String> {
    let contents = fs::read_to_string(path)?;
    let docs = YamlLoader::load_from_str(&contents)?;
    let doc = &docs[0];

    let original = doc.as_hash().unwrap();
    let mut hash = LinkedHashMap::<Yaml, Yaml>::new();

    for (key, value) in original {
        let key_str = key.as_str().unwrap();
        if keys_to_strip.contains(&key_str) {
            continue;
        }
        hash.insert(key.clone(), value.clone());
    }

    let mut output = String::new();

    let new_yaml = yaml::Yaml::Hash(hash);

    let mut emitter = YamlEmitter::new(&mut output);

    emitter.dump(&new_yaml).unwrap(); // dump the YAML object to a String

    Ok(output)
}
