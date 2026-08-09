use std::path::PathBuf;

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub enum JavaDiagnosticKind {
    #[default]
    Error,
    Warning,
}

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct JavaDiagnostic {
    pub kind: JavaDiagnosticKind,
    pub file: PathBuf,
    pub line_number: usize,
    pub line_text: String,
    pub column: usize,
    pub message: String,
    pub hints: Vec<String>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum JavaDiagnosticParserStage {
    Gathering,
    LineText,
    ColumnMarker,
    Hints,
    EndOfFile,
}

pub fn parse(input: &str) -> anyhow::Result<Vec<JavaDiagnostic>> {
    const ERROR_TEXT: &str = ": error: ";
    const WARNING_TEXT: &str = ": warning: ";

    let mut out = vec![];

    let mut diagnostic = JavaDiagnostic::default();
    let mut stage = JavaDiagnosticParserStage::Gathering;

    for line in input.lines() {
        if stage == JavaDiagnosticParserStage::Hints {
            let error = line.find(": error: ");
            let warning = line.find(": warning: ");

            if error.is_some() || warning.is_some() {
                out.push(diagnostic);
                diagnostic = JavaDiagnostic::default();
                stage = JavaDiagnosticParserStage::Gathering;
            }

            if line.ends_with(" errors")
                || line.ends_with(" warnings")
                || line.ends_with(" error")
                || line.ends_with(" warning")
            {
                out.push(diagnostic);
                diagnostic = JavaDiagnostic::default();
                stage = JavaDiagnosticParserStage::EndOfFile;
            }
        }

        match stage {
            JavaDiagnosticParserStage::Gathering => {
                let error = line.find(": error: ");
                let warning = line.find(": warning: ");

                let kind =
                    warning.map_or(JavaDiagnosticKind::Error, |_| JavaDiagnosticKind::Warning);

                let start = if kind == JavaDiagnosticKind::Error {
                    error
                } else {
                    warning
                }
                .ok_or_else(|| {
                    anyhow::anyhow!("expected 'error' or 'warning' but didn't find neither")
                })?;
                let error_text = if kind == JavaDiagnosticKind::Error {
                    ERROR_TEXT
                } else {
                    WARNING_TEXT
                };

                let filename_with_line_number = &line[..start];
                let split = filename_with_line_number.split(':').collect::<Vec<&str>>();
                let filename = split[..split.len() - 1].join(":");

                let line_number = filename_with_line_number
                    .split(':')
                    .next_back()
                    .ok_or_else(|| anyhow::anyhow!("expected line number but got nothing"))?;

                let message = &line[(start + error_text.len())..];

                diagnostic.kind = kind;
                diagnostic.file = PathBuf::from(filename).canonicalize()?;
                diagnostic.line_number = line_number.parse()?;
                diagnostic.message = message.to_string();

                stage = JavaDiagnosticParserStage::LineText;
            }
            JavaDiagnosticParserStage::LineText => {
                diagnostic.line_text = line.to_string();
                stage = JavaDiagnosticParserStage::ColumnMarker;
            }
            JavaDiagnosticParserStage::ColumnMarker => {
                diagnostic.column = line.len();
                stage = JavaDiagnosticParserStage::Hints;
            }
            JavaDiagnosticParserStage::Hints => {
                diagnostic.hints.push(line.to_string());
            }
            JavaDiagnosticParserStage::EndOfFile => {}
        }
    }

    Ok(out)
}

#[cfg(test)]
mod test {
    use super::*;

    fn fd(
        kind: JavaDiagnosticKind,
        file: &str,
        line_number: usize,
        line_text: &str,
        column: usize,
        message: &str,
        hints: &[&str],
    ) -> JavaDiagnostic {
        JavaDiagnostic {
            kind,
            file: file.into(),
            line_text: line_text.to_string(),
            message: message.to_string(),
            line_number,
            column,
            hints: hints.iter().map(std::string::ToString::to_string).collect(),
        }
    }

    #[test]
    fn simple() {
        assert_eq!(
                    parse(
                        r"G:\some\build\path\src\main\java\com\example\Main.java:9: error: cannot find symbol
        test.testaaaaa();
            ^
symbol:   method testaaaaa()
location: variable test of type List
1 error"
                    ).unwrap(),
                    vec![fd(
                        JavaDiagnosticKind::Error,
                        r"G:\some\build\path\src\main\java\com\example\Main.java",
                        9,
                        "        test.testaaaaa();",
                        13,
                        "cannot find symbol",
                        &[
                            "symbol:   method testaaaaa()",
                            "location: variable test of type List"
                        ]
                    )]
                );

        assert_eq!(
                    parse(
                        r"G:\some\build\path\src\main\java\com\example\Main.java:9: warning: [rawtypes] found raw type: ArrayList
        List test = new ArrayList();
                        ^
  missing type arguments for generic class ArrayList<E>
  where E is a type-variable:
    E extends Object declared in class ArrayList
1 warning"
                    ).unwrap(),
                    vec![fd(
                        JavaDiagnosticKind::Warning,
                        r"G:\some\build\path\src\main\java\com\example\Main.java",
                        9,
                        "        List test = new ArrayList();",
                        25,
                        "[rawtypes] found raw type: ArrayList",
                        &[
                            "missing type arguments for generic class ArrayList<E>",
                            "where E is a type-variable:",
                            "E extends Object declared in class ArrayList",
                        ]
                    )]
                );

        // UNIX paths
        assert_eq!(
            parse(
                r"/home/build/path/src/main/java/com/example/Main.java:9: error: cannot find symbol
        test.testaaaaa();
            ^
symbol:   method testaaaaa()
location: variable test of type List
1 error"
            )
            .unwrap(),
            vec![fd(
                JavaDiagnosticKind::Error,
                r"/home/build/path/src/main/java/com/example/Main.java",
                9,
                "        test.testaaaaa();",
                13,
                "cannot find symbol",
                &[
                    "symbol:   method testaaaaa()",
                    "location: variable test of type List"
                ]
            )]
        );

        // long paths
        assert_eq!(
                    parse(
                        r"G:\some\build\path\src\main\java\com\example\Looongname.java:923: error: cannot find symbol
        test.testaaaaa();
            ^
symbol:   method testaaaaa()
location: variable test of type List
1 error"
                    ).unwrap(),
                    vec![fd(
                        JavaDiagnosticKind::Error,
                        r"G:\some\build\path\src\main\java\com\example\Looongname.java",
                        923,
                        "        test.testaaaaa();",
                        13,
                        "cannot find symbol",
                        &[
                            "symbol:   method testaaaaa()",
                            "location: variable test of type List"
                        ]
                    )]
                );
    }

    #[test]
    fn multiple() {
        assert_eq!(
                    parse(
                        r"G:\some\build\path\src\main\java\com\example\Main.java:9: error: cannot find symbol
        test.testaaaaa();
            ^
symbol:   method testaaaaa()
location: variable test of type List
G:\some\build\path\src\main\java\com\example\Test.java:162: error: cannot find symbol
        gas.some();
           ^
symbol:   method some()
location: variable gas of type ArrayList
2 errors"
                    ).unwrap(),
                    vec![fd(
                        JavaDiagnosticKind::Error,
                        r"G:\some\build\path\src\main\java\com\example\Main.java",
                        9,
                        "        test.testaaaaa();",
                        13,
                        "cannot find symbol",
                        &[
                            "symbol:   method testaaaaa()",
                            "location: variable test of type List"
                        ]
                    ),
                    fd(
                        JavaDiagnosticKind::Error,
                        r"G:\some\build\path\src\main\java\com\example\Test.java",
                        162,
                        "        gas.some();",
                        12,
                        "cannot find symbol",
                        &[
                            "symbol:   method some()",
                            "location: variable gas of type ArrayList"
                        ]
                    )
                    ]
                );
    }

    #[test]
    fn complex() {
        assert_eq!(
                    parse(
                        r"G:\some\build\path\src\main\java\com\example\Test.java:8: warning: [rawtypes] found raw type: List
        List test = new ArrayList();
        ^
  missing type arguments for generic class List<E>
  where E is a type-variable:
    E extends Object declared in interface List
G:\some\build\path\src\main\java\com\example\Test.java:8: warning: [rawtypes] found raw type: ArrayList
        List test = new ArrayList();
                        ^
  missing type arguments for generic class ArrayList<E>
  where E is a type-variable:
    E extends Object declared in class ArrayList
G:\some\build\path\src\main\java\com\example\Test.java:9: error: cannot find symbol
        test.testaaaaa();
            ^
  symbol:   method testaaaaa()
  location: variable test of type List
1 error
2 warnings"
                    ).unwrap(),
                    vec![
                        fd(
                            JavaDiagnosticKind::Warning,
                            r"G:\some\build\path\src\main\java\com\example\Test.java",
                            8,
                            "        List test = new ArrayList();",
                            9,
                            "[rawtypes] found raw type: List",
                            &[
                                "missing type arguments for generic class List<E>",
                                "where E is a type-variable:",
                                "E extends Object declared in interface List",
                            ]
                        ),
                        fd(
                            JavaDiagnosticKind::Warning,
                            r"G:\some\build\path\src\main\java\com\example\Test.java",
                            8,
                            "        List test = new ArrayList();",
                            25,
                            "[rawtypes] found raw type: ArrayList",
                            &[
                                "missing type arguments for generic class ArrayList<E>",
                                "where E is a type-variable:",
                                "E extends Object declared in class ArrayList",
                            ]
                        ),
                        fd(
                            JavaDiagnosticKind::Error,
                            r"G:\some\build\path\src\main\java\com\example\Test.java",
                            9,
                            "        test.testaaaaa();",
                            13,
                            "cannot find symbol",
                            &[
                                "symbol:   method testaaaaa()",
                                "location: variable test of type List",
                            ]
                        )
                    ]
                );
    }
}
