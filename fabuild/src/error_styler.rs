use std::{collections::HashMap, io::Write, path::PathBuf};

use bhc_diagnostics::{Diagnostic, Severity, SourceMap};
use bhc_span::{FileId, FullSpan, Span};

use crate::error_parser::{JavaDiagnostic, JavaDiagnosticKind};

fn line_col_to_byte_index(source: &str, line: usize, col: usize) -> Option<usize> {
    if line == 0 || col == 0 {
        return None;
    }

    let mut current_line = 1;
    let mut byte_offset = 0;

    for (i, c) in source.char_indices() {
        if current_line == line {
            break;
        }
        if c == '\n' {
            current_line += 1;
            byte_offset = i + 1;
        }
    }

    if current_line < line {
        return None;
    }

    let target_str = &source[byte_offset..];
    let mut char_count = 0;
    let mut col_byte_offset = 0;

    for (i, c) in target_str.char_indices() {
        if char_count == col - 1 || c == '\n' {
            break;
        }
        char_count += 1;
        col_byte_offset = i + c.len_utf8();
    }

    if char_count < col - 1 {
        return None;
    }

    Some(byte_offset + col_byte_offset)
}

pub fn print_pretty_error<S: ::std::hash::BuildHasher>(
    source_map: &SourceMap,
    file_ids: &HashMap<PathBuf, FileId, S>,
    parsed: &[JavaDiagnostic],
) -> anyhow::Result<()> {
    let mut dgs = vec![];

    for diagnostic in parsed {
        let file_id = *file_ids.get(&diagnostic.file).ok_or_else(|| {
            anyhow::anyhow!("file {} is not in file id list?", diagnostic.file.display())
        })?;
        let file = source_map.get_file(file_id).ok_or_else(|| {
            anyhow::anyhow!("file {} is not in source map?", diagnostic.file.display())
        })?;
        let line_start = line_col_to_byte_index(&file.src, diagnostic.line_number, 1)
            .ok_or_else(|| anyhow::anyhow!("parser lead to non-existing line number?"))?;
        let start = line_start
            + file.src[line_start..]
                .find(|c: char| !c.is_whitespace())
                .ok_or_else(|| anyhow::anyhow!("parser lead to whitespace?"))?;
        let end = start
            + file.src[start..]
                .find('\n')
                .unwrap_or_else(|| file.src[start..].len());

        let span = FullSpan::new(
            file_id,
            Span::from_raw(u32::try_from(start)?, u32::try_from(end)?),
        );

        let mut diag = if diagnostic.kind == JavaDiagnosticKind::Error {
            Diagnostic::error(&diagnostic.message)
        } else {
            Diagnostic::warning(&diagnostic.message)
        };
        diag = diag.with_label(span, "");

        for hint in &diagnostic.hints {
            diag = diag.with_note(hint);
        }

        dgs.push(diag);
    }

    let renderer = PrettyDiagnosticRenderer::new(source_map);
    renderer.render_all(&dgs);

    Ok(())
}

/// Render diagnostics to a writer.
pub struct PrettyDiagnosticRenderer<'a> {
    source_map: &'a SourceMap,
}

impl<'a> PrettyDiagnosticRenderer<'a> {
    /// Create a new renderer.
    #[must_use]
    pub const fn new(source_map: &'a SourceMap) -> Self {
        Self { source_map }
    }

    /// Render a diagnostic to the given writer.
    pub fn render(&self, diagnostic: &Diagnostic, w: &mut impl Write) -> std::io::Result<()> {
        let reset = "\x1b[0m";
        let color = diagnostic.severity.color();

        // Header
        write!(w, "{}{}", color, diagnostic.severity.label())?;
        if let Some(code) = &diagnostic.code {
            write!(w, "[{code}]")?;
        }
        writeln!(w, "{reset}: {}", diagnostic.message)?;

        // Labels
        for label in &diagnostic.labels {
            if let Some(file) = self.source_map.get_file(label.span.file) {
                let loc = file.lookup_line_col(label.span.span.lo);
                let arrow = if label.primary { "-->" } else { "   " };
                writeln!(
                    w,
                    " {}{arrow}{reset} {}:{}:{}",
                    Severity::Note.color(),
                    file.name,
                    loc.line,
                    loc.col
                )?;

                // Show source line
                if !label.span.span.is_dummy() {
                    let source = file.source_text(label.span.span);
                    writeln!(w, "   {}|{reset}", Severity::Note.color())?;
                    writeln!(w, "   {}|{reset} {source}", Severity::Note.color())?;
                    writeln!(
                        w,
                        "   {}|{reset} {}{}{reset}",
                        Severity::Note.color(),
                        color,
                        "^".repeat(source.len().max(1))
                    )?;
                    if !label.message.is_empty() {
                        writeln!(w, "   | {}", label.message)?;
                    }
                }
            }
        }

        // Notes
        for note in &diagnostic.notes {
            writeln!(w, " {}= note{reset}: {note}", Severity::Note.color())?;
        }

        // Suggestions
        for suggestion in &diagnostic.suggestions {
            writeln!(
                w,
                " = {}help{reset}: {}",
                Severity::Help.color(),
                suggestion.message
            )?;
            if !suggestion.replacement.is_empty() {
                writeln!(w, "   {}|{reset}", Severity::Note.color())?;
                writeln!(
                    w,
                    "   {}|{reset} {}",
                    Severity::Note.color(),
                    suggestion.replacement
                )?;
            }
        }

        writeln!(w)?;
        Ok(())
    }

    /// Render all diagnostics to stderr.
    pub fn render_all(&self, diagnostics: &[Diagnostic]) {
        let mut stderr = std::io::stderr().lock();

        for diag in diagnostics {
            let _ = self.render(diag, &mut stderr);
        }
    }
}
