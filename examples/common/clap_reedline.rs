//! A `reedline` loop that parses each line as a `clap` command.
//!
//! Stands in for the `clap-repl` crate, which pinned `reedline 0.38` and with
//! it the unmaintained `chrono` (`RUSTSEC-2026-0543`). Shared by `kuhn_repl`,
//! `dealer_repl` and `table0` through `#[path = "common/clap_reedline.rs"]`.
//!
//! Lines are split on whitespace, so arguments cannot hold spaces.

use clap::{CommandFactory, Parser};
use reedline::{
    Completer, CompletionResult, DefaultPrompt, DefaultPromptSegment, Emacs, FileBackedHistory, IdeMenu, KeyCode,
    KeyModifiers, MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal, Span, Suggestion,
    default_emacs_keybindings,
};
use std::marker::PhantomData;
use std::path::PathBuf;

const COMPLETION_MENU: &str = "completion_menu";

/// Reads lines, parses each one as a `C`, and hands it to a handler.
pub struct ClapReedline<C> {
    editor: Reedline,
    prompt: DefaultPrompt,
    command: PhantomData<C>,
}

impl<C: Parser> ClapReedline<C> {
    /// Builds the editor with Tab completion of command names and a history
    /// file. If the history file cannot be opened, history stays in memory.
    pub fn new(prompt_name: &str, history_file: &str, history_size: usize) -> Self {
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Tab,
            ReedlineEvent::UntilFound(vec![
                ReedlineEvent::Menu(COMPLETION_MENU.to_owned()),
                ReedlineEvent::MenuNext,
            ]),
        );
        let menu = IdeMenu::default().with_default_border().with_name(COMPLETION_MENU);
        let history = FileBackedHistory::with_file(history_size, PathBuf::from(history_file)).unwrap_or_default();
        let editor = Reedline::create()
            .with_completer(Box::new(CommandNameCompleter::of::<C>()))
            .with_menu(ReedlineMenu::EngineCompleter(Box::new(menu)))
            .with_edit_mode(Box::new(Emacs::new(keybindings)))
            .with_history(Box::new(history));
        let prompt = DefaultPrompt {
            left_prompt: DefaultPromptSegment::Basic(prompt_name.to_owned()),
            ..DefaultPrompt::default()
        };
        Self {
            editor,
            prompt,
            command: PhantomData,
        }
    }

    /// Runs until Ctrl-D. Ctrl-C clears the line. A line that does not parse
    /// prints clap's error and the loop goes on.
    pub fn repl(mut self, mut handler: impl FnMut(C)) {
        loop {
            match self.editor.read_line(&self.prompt) {
                Ok(Signal::Success(line)) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    match C::try_parse_from(std::iter::once("").chain(line.split_whitespace())) {
                        Ok(command) => handler(command),
                        Err(error) => {
                            let _ = error.print();
                        }
                    }
                }
                Ok(Signal::CtrlD) => break,
                Ok(_) => {}
                Err(error) => {
                    eprintln!("✗ terminal error: {error}");
                    break;
                }
            }
        }
    }
}

/// Completes the first word of a line from the command's subcommand names.
struct CommandNameCompleter {
    names: Vec<(String, Option<String>)>,
}

impl CommandNameCompleter {
    fn of<C: CommandFactory>() -> Self {
        let mut names: Vec<(String, Option<String>)> = C::command()
            .get_subcommands()
            .map(|sub| (sub.get_name().to_owned(), sub.get_about().map(ToString::to_string)))
            .collect();
        names.push(("help".to_owned(), Some("Print help".to_owned())));
        Self { names }
    }
}

impl Completer for CommandNameCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> CompletionResult {
        let typed = line.get(..pos).unwrap_or(line);
        let start = typed.len() - typed.trim_start().len();
        let word = &typed[start..];
        if word.contains(char::is_whitespace) {
            return CompletionResult::fresh(Vec::new());
        }
        let suggestions: Vec<Suggestion> = self
            .names
            .iter()
            .filter(|(name, _)| name.starts_with(word))
            .map(|(name, about)| Suggestion {
                value: name.clone(),
                description: about.clone(),
                span: Span::new(start, pos),
                append_whitespace: true,
                ..Suggestion::default()
            })
            .collect();
        CompletionResult::fresh(suggestions)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod examples__common__clap_reedline_tests {
    use super::*;

    #[derive(Debug, Parser)]
    #[command(name = "")]
    enum Sample {
        /// Show chips.
        Chips,
        Check,
        Bet {
            amount: usize,
        },
    }

    impl Sample {
        fn values(line: &str) -> Vec<String> {
            CommandNameCompleter::of::<Sample>()
                .complete(line, line.len())
                .suggestions()
                .iter()
                .map(|suggestion| suggestion.value.clone())
                .collect()
        }
    }

    #[test]
    fn completer_offers_names_that_start_with_the_typed_word() {
        assert_eq!(Sample::values("ch"), vec!["chips", "check"]);
    }

    #[test]
    fn completer_replaces_only_the_typed_word() {
        let result = CommandNameCompleter::of::<Sample>().complete("  chi", 5);
        let suggestion = &result.suggestions()[0];
        assert_eq!(suggestion.value, "chips");
        assert_eq!(suggestion.span, Span::new(2, 5));
        assert_eq!(suggestion.description.as_deref(), Some("Show chips"));
    }

    #[test]
    fn completer_offers_every_name_and_help_on_an_empty_line() {
        assert_eq!(Sample::values(""), vec!["chips", "check", "bet", "help"]);
    }

    #[test]
    fn completer_stays_quiet_after_the_first_word() {
        assert!(Sample::values("bet 1").is_empty());
    }

    #[test]
    fn parsed_line_ignores_extra_whitespace() {
        let parsed = Sample::try_parse_from(std::iter::once("").chain("  bet   40 ".split_whitespace()));
        assert!(matches!(parsed, Ok(Sample::Bet { amount: 40 })));
    }
}
