use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;

use anyhow::Context;

use crate::project::GrustleProjectTree;

pub fn is_positive(text: &str) -> anyhow::Result<bool> {
    if text == "0" || text == "false" {
        return Ok(false);
    }

    if text == "1" || text == "true" {
        return Ok(true);
    }

    anyhow::bail!("boolean value is not valid: '{text}'")
}

#[allow(clippy::too_many_lines)]
pub fn preprocess_source_file(tree: &GrustleProjectTree, path: &Path) -> anyhow::Result<String> {
    let filename = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("filename is non UTF-8"))?
        .to_string_lossy()
        .to_string();
    let content = std::fs::read_to_string(path)?;
    let mut out = String::new();

    let mut can_emit = 0;
    let mut inside_conditional_block = false;
    let mut passed_conditional_block = false;

    let mut defines = HashMap::new();
    defines.insert("GRUSTLE".to_owned(), "true".to_owned());

    for (line_index, line) in content
        .split('\n')
        .map(std::string::ToString::to_string)
        .collect::<Vec<String>>()
        .iter()
        .enumerate()
    {
        let line_number = line_index + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("// #") || trimmed.starts_with("//#") {
            // assume this is a preprocessor directive
            let line = trimmed
                .trim_start_matches("// #")
                .trim_start_matches("//#")
                .to_string();
            let mut split = line.split(' ');
            let directive = split
                .next()
                .ok_or_else(|| anyhow::anyhow!("no directive was specified"))?;

            macro_rules! expect_next {
                () => {
                    split
                        .next()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "incomplete directive {directive}, expected another parameter"
                            )
                        })?
                        .to_string()
                };
            }

            macro_rules! expect_next_or {
                ($or:expr) => {
                    split.next().unwrap_or($or).to_string()
                };
            }

            match directive {
                "if" => {
                    // TODO: version(minecraft) >= 26.0
                    let full_condition = expect_next!();
                    let function = full_condition
                        .split('(')
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("missing function name"))?;
                    let args = full_condition
                        .split(')')
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("missing function name"))?
                        .split('(')
                        .nth(1)
                        .ok_or_else(|| anyhow::anyhow!("missing function args"))?;
                    let operator = expect_next_or!("==");
                    let value = expect_next_or!("1");

                    match function {
                        "version" => {
                            let package = args;
                            let ver = tree.packages.iter().find(|p| p.get_full_name() == package).ok_or_else(|| anyhow::anyhow!("failed to find package {package} (from {filename}:L{line_number})"))?.package.ver.clone();
                            let pkg_version = lenient_semver::parse(
                                // I sure do hope this memory doesn't leak!
                                // inconspicious memory leak:
                                ver.leak()
                            ).context(format!("failed to parse pkg version from package {package} (from {filename}:L{line_number})"))?;
                            let value_version = lenient_semver::parse(
                                // I sure do hope this memory doesn't leak!
                                // inconspicious memory leak:
                                value.leak(),
                            )
                            .context(format!(
                                "failed to parse value as version (from {filename}:L{line_number})"
                            ))?;

                            let passes = match operator.as_str() {
                                ">=" => pkg_version >= value_version,
                                "<=" => pkg_version <= value_version,
                                ">" => pkg_version > value_version,
                                "<" => pkg_version < value_version,
                                "==" => pkg_version == value_version,
                                _ => {
                                    anyhow::bail!(
                                        "unknown #if version operator \"{operator}\" (from {filename}:L{line_number})"
                                    )
                                }
                            };

                            if passes {
                                passed_conditional_block = true;
                            } else {
                                can_emit -= 1;
                                inside_conditional_block = true;
                                passed_conditional_block = false;
                            }
                        }
                        _ => {
                            anyhow::bail!(
                                "unknown #if function \"{function}\" (from {filename}:L{line_number})"
                            );
                        }
                    }
                }
                "ifdef" => {
                    let key = expect_next!();

                    if !defines.contains_key(&key)
                        || !is_positive(defines.get(&key).ok_or_else(|| unreachable!())?)?
                    {
                        can_emit -= 1;
                        inside_conditional_block = true;
                        passed_conditional_block = false;
                    } else {
                        passed_conditional_block = true;
                    }
                }
                "endif" => {
                    if inside_conditional_block {
                        can_emit += 1;
                        inside_conditional_block = false;
                    }

                    if passed_conditional_block {
                        passed_conditional_block = false;
                    }
                }
                "define" => {
                    let key = expect_next!();
                    let value = expect_next_or!("1");
                    defines.insert(key, value);
                }
                _ => {
                    anyhow::bail!(
                        "unknown directive: \"{directive}\" (from {filename}:L{line_number})"
                    );
                }
            }
        } else if can_emit >= 0 {
            let mut line = line.clone();

            if passed_conditional_block {
                line = line.trim_start_matches("//").to_string();
            }

            writeln!(&mut out, "{line}")?;
        }
    }

    anyhow::ensure!(can_emit == 0, "unescaped condition in {filename}");

    Ok(out)
}
