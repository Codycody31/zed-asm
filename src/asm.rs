use std::fs;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct AsmExtension {
    cached_binary_path: Option<String>,
}

impl AsmExtension {
    fn language_server_binary_path(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String, String> {
        // Check if `asm-lsp` is available in the PATH
        if let Some(path) = worktree.which("asm-lsp") {
            return Ok(path);
        }

        // TODO: If not found, try to download it from the repo (cloned version)

        Err("asm-lsp binary not found in the system PATH".into())
    }

    fn extract_comment_above_function<P: AsRef<Path>>(
        &self,
        path: P,
        line_number: usize,
    ) -> Option<String> {
        let file = File::open(path).ok()?;
        let lines: Vec<String> = io::BufReader::new(file)
            .lines()
            .filter_map(Result::ok)
            .collect();

        if line_number == 0 || line_number > lines.len() {
            return None;
        }

        let mut comments = Vec::new();
        for i in (0..line_number).rev() {
            let line = &lines[i];
            if line.trim().starts_with(';') {
                comments.push(line.trim_start_matches(';').trim().to_string());
            } else if !line.trim().is_empty() {
                break;
            }
        }

        if comments.is_empty() {
            None
        } else {
            Some(comments.into_iter().rev().collect::<Vec<_>>().join("\n"))
        }
    }
}

impl zed::Extension for AsmExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id, worktree)?,
            args: Default::default(),
            env: Default::default(),
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: zed::lsp::Completion,
    ) -> Option<zed::CodeLabel> {
        // Adjusting for typical assembly completions, such as instructions, registers, or labels
        let label_kind = match completion.kind? {
            zed::lsp::CompletionKind::Keyword => "keyword",
            zed::lsp::CompletionKind::Variable => "variable",
            zed::lsp::CompletionKind::Function => "function",
            _ => "other",
        };

        Some(zed::CodeLabel {
            spans: vec![zed::CodeLabelSpan::literal(
                completion.label.clone(),
                Some(label_kind.into()),
            )],
            filter_range: (0..completion.label.len()).into(),
            code: completion.label,
        })
    }

    fn label_for_symbol(
        &self,
        _language_server_id: &LanguageServerId,
        symbol: zed::lsp::Symbol,
    ) -> Option<zed::CodeLabel> {
        // Adjusting for typical assembly symbols like functions, macros, labels, etc.
        let symbol_kind = match symbol.kind {
            zed::lsp::SymbolKind::Function => "Function",
            zed::lsp::SymbolKind::Variable => "Variable",
            zed::lsp::SymbolKind::Constant => "Constant",
            // zed::lsp::SymbolKind::Macro => "Macro",
            _ => "Other",
        };

        let code = format!("{}: {}", symbol_kind, symbol.name);

        Some(zed::CodeLabel {
            spans: vec![zed::CodeLabelSpan::literal(
                symbol.name.clone(),
                Some(symbol_kind.into()),
            )],
            filter_range: (0..symbol.name.len()).into(),
            code,
        })
    }

    // TODO: on hover of macro, show content of comment directly above it
    // TODO: Autocomplete for things like "SYS_", etc when things such as "SYS_CLOSE   equ 3" are defined
    // NOTE: The above two features are not implemented yet, and may actually require a custom language server
}

zed::register_extension!(AsmExtension);
