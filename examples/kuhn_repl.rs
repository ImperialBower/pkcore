//! Interactive REPL for Kuhn poker — play against the GTO strategy.
//!
//! You are Player 0 (first to act). The GTO opponent plays Player 1
//! using the analytical Nash equilibrium strategy with `alpha = 1/3`
//! (the maximum-bluff Nash equilibrium).
//!
//! The game value for Player 0 is **−1/18 ≈ −0.0556 chips per hand**.
//! That means GTO wins from P1's seat in the long run — use `hint` to
//! see what GTO would do in your position.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example kuhn_repl
//! ```
//!
//! # Quick-start session
//!
//! ```text
//! kuhn❯ deal
//! kuhn❯ hint        # see GTO probabilities for your card + history
//! kuhn❯ check
//! kuhn❯ deal
//! kuhn❯ bet
//! kuhn❯ stats
//! kuhn❯ quit
//! ```

use clap::Parser;
use clap_repl::ClapEditor;
use pkcore::games::kuhn::{KuhnAction, KuhnCard, KuhnState, KuhnStrategy};
use rand::Rng;
use rand::prelude::IndexedRandom;
use reedline::{DefaultPrompt, DefaultPromptSegment, FileBackedHistory};

// ── All possible Kuhn deals (ordered pairs of distinct cards) ─────────────────

const DEALS: [(KuhnCard, KuhnCard); 6] = [
    (KuhnCard::Jack, KuhnCard::Queen),
    (KuhnCard::Jack, KuhnCard::King),
    (KuhnCard::Queen, KuhnCard::Jack),
    (KuhnCard::Queen, KuhnCard::King),
    (KuhnCard::King, KuhnCard::Jack),
    (KuhnCard::King, KuhnCard::Queen),
];

// ── Session state ─────────────────────────────────────────────────────────────

struct Session {
    /// Current game state, if a hand is in progress.
    state: Option<KuhnState>,
    /// The GTO Nash equilibrium strategy (alpha = 1/3).
    strategy: KuhnStrategy,
    /// Random number generator.
    rng: rand::rngs::ThreadRng,
    /// Total hands played.
    hands: u32,
    /// Cumulative chip delta from Player 0's perspective.
    net: i32,
    /// P0's hole card for the current hand.
    p0_card: Option<KuhnCard>,
    /// P1's hole card (revealed at showdown).
    p1_card: Option<KuhnCard>,
}

impl Session {
    fn new() -> Self {
        Session {
            state: None,
            strategy: KuhnStrategy::default(),
            rng: rand::rng(),
            hands: 0,
            net: 0,
            p0_card: None,
            p1_card: None,
        }
    }

    /// Randomly deals a new hand from the 6 possible distinct-card pairs.
    fn deal(&mut self) {
        let &(c0, c1) = DEALS.choose(&mut self.rng).expect("DEALS is non-empty");
        // KuhnState::new cannot fail for distinct cards.
        self.state = Some(KuhnState::new(c0, c1).expect("distinct cards"));
        self.p0_card = Some(c0);
        self.p1_card = Some(c1);
        self.hands += 1;
    }

    /// Advances the game by sampling and applying GTO actions for P1 until it
    /// is the human's turn (P0) again or the hand reaches a terminal node.
    fn gto_respond(&mut self) {
        loop {
            let state = match self.state.clone() {
                Some(s) if !s.is_terminal() => s,
                _ => break,
            };
            // Only act on P1's turn.
            if state.current_player() != Some(1) {
                break;
            }
            let info = state.info_set(1);
            let probs = self.strategy.action_probs(&info);

            // Sample an action proportional to the GTO probabilities.
            let r: f64 = self.rng.random();
            let mut cumulative = 0.0;
            let mut chosen = probs[0].0;
            for &(action, p) in probs {
                cumulative += p;
                if r < cumulative {
                    chosen = action;
                    break;
                }
            }

            println!("  GTO (P1) → {chosen}");
            let next = state.apply(chosen).expect("GTO action is legal");
            if next.is_terminal() {
                self.finish(next);
                return;
            }
            self.state = Some(next);
        }
    }

    /// Prints the hand result and resets the state.
    fn finish(&mut self, terminal: KuhnState) {
        let payoff = terminal.payoff().expect("terminal state has a payoff");
        let delta = payoff[0];
        self.net += delta;

        let result_line = if delta > 0 {
            format!("You win +{delta} chip(s)")
        } else if delta < 0 {
            format!("You lose {} chip(s)", delta.abs())
        } else {
            "Chop (0)".to_owned()
        };

        println!();
        println!("  ╔═══ Hand Result ══════════════════════════╗");
        println!("  ║  Your card  : {}", self.p0_card.unwrap());
        println!("  ║  GTO card   : {}", self.p1_card.unwrap());
        println!("  ║  History    : {}", terminal.history());
        println!("  ║  Outcome    : {result_line}");
        println!("  ║  Net total  : {:+}  ({} hands)", self.net, self.hands);
        println!("  ╚══════════════════════════════════════════╝");
        println!();

        self.state = None;
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Kuhn poker — play against the GTO Nash equilibrium strategy.
#[derive(Debug, Parser)]
#[command(name = "")]
enum Command {
    /// Deal a new hand. You are Player 0; GTO plays Player 1.
    #[command(alias = "d")]
    Deal,

    /// Check — pass the action without betting.
    #[command(alias = "c")]
    Check,

    /// Bet 1 chip.
    #[command(alias = "b")]
    Bet,

    /// Fold — surrender the pot to your opponent.
    #[command(alias = "f")]
    Fold,

    /// Call the opponent's bet.
    Call,

    /// Show the GTO probability distribution for your current info set.
    ///
    /// This tells you what the Nash equilibrium strategy says to do with
    /// your card in the current position. Use it to learn optimal play.
    #[command(alias = "h")]
    Hint,

    /// Show the current game state (your card, history, legal actions).
    #[command(alias = "s")]
    Status,

    /// Show cumulative chip statistics.
    Stats,

    /// Exit the REPL.
    #[command(alias = "q")]
    Quit,
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║              Kuhn Poker REPL                     ║");
    println!("║  You are Player 0  ·  GTO plays Player 1         ║");
    println!("║  Cards: Jack < Queen < King  ·  Ante: 1 chip     ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("  Commands: deal · check · bet · fold · call");
    println!("            hint (GTO advice) · status · stats · quit");
    println!("  Aliases : d    · c     · b   · f    · (full)");
    println!();

    let mut session = Session::new();

    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("kuhn".to_owned()),
        ..DefaultPrompt::default()
    };

    let rl = ClapEditor::<Command>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(|reed| {
            reed.with_history(Box::new(
                FileBackedHistory::with_file(1_000, "./generated/kuhn-repl-history".into()).unwrap_or_default(),
            ))
        })
        .build();

    rl.repl(|cmd| handle(&mut session, cmd));
}

// ── Command dispatch ──────────────────────────────────────────────────────────

fn handle(session: &mut Session, command: Command) {
    match command {
        Command::Deal => {
            if session.state.is_some() {
                println!("  ✗ A hand is already in progress. Finish it first.");
                return;
            }
            session.deal();
            let card = session.p0_card.unwrap();
            println!();
            println!("  New hand dealt.");
            println!("  Your card : [{card}]  (GTO's card is hidden)");
            println!("  Pot       : 2 chips (antes posted)");
            println!("  You are first to act — legal: check, bet");
            println!("  (Type 'hint' to see GTO frequencies for your hand)");
            println!();
        }

        Command::Check => apply_human(session, KuhnAction::Check),
        Command::Bet => apply_human(session, KuhnAction::Bet),
        Command::Fold => apply_human(session, KuhnAction::Fold),
        Command::Call => apply_human(session, KuhnAction::Call),

        Command::Hint => {
            let Some(state) = &session.state else {
                println!("  No hand in progress. Type 'deal' to start.");
                return;
            };
            let Some(0) = state.current_player() else {
                println!("  It is not your turn.");
                return;
            };
            let card = session.p0_card.unwrap();
            let info = state.info_set(0);
            let probs = session.strategy.action_probs(&info);
            println!();
            println!("  GTO frequencies for [{card}] | history: {}:", state.history());
            for &(action, p) in probs {
                let filled = (p * 20.0).round() as usize;
                let bar = format!("{}{}", "#".repeat(filled), ".".repeat(20 - filled));
                println!("    {action:<8}  {:5.1}%  |{bar}|", p * 100.0);
            }
            println!();
        }

        Command::Status => match &session.state {
            None => println!("  No hand in progress. Type 'deal' to start."),
            Some(state) => {
                let card = session.p0_card.unwrap();
                let actions: Vec<String> = state.legal_actions().iter().map(|a| a.to_string()).collect();
                let whose_turn = match state.current_player() {
                    Some(0) => "You (P0)".to_owned(),
                    Some(1) => "GTO (P1)".to_owned(),
                    _ => "Terminal".to_owned(),
                };
                println!();
                println!("  Your card : [{card}]  (GTO's card is hidden)");
                println!("  History   : {}", state.history());
                println!("  To act    : {whose_turn}");
                println!("  Legal     : {}", actions.join(", "));
                println!();
            }
        },

        Command::Stats => {
            println!();
            println!("  ┌─ Statistics ─────────────────────────────┐");
            println!("  │  Hands played  : {}", session.hands);
            println!("  │  Net chips     : {:+}", session.net);
            if session.hands > 0 {
                let avg = session.net as f64 / session.hands as f64;
                println!("  │  Average/hand  : {avg:+.4}");
                println!("  │  GTO game value for P0: −0.0556 chips/hand");
                println!("  │  (P0 is at a structural disadvantage in Kuhn)");
            }
            println!("  └──────────────────────────────────────────┘");
            println!();
        }

        Command::Quit => {
            println!("Goodbye!");
            std::process::exit(0);
        }
    }
}

/// Applies a human (P0) action, then triggers GTO's response.
fn apply_human(session: &mut Session, action: KuhnAction) {
    let Some(state) = &session.state else {
        println!("  ✗ No hand in progress. Type 'deal' first.");
        return;
    };
    if state.current_player() != Some(0) {
        println!("  ✗ Not your turn.");
        return;
    }
    if !state.legal_actions().contains(&action) {
        let legal: Vec<String> = state.legal_actions().iter().map(|a| a.to_string()).collect();
        println!("  ✗ '{action}' is not legal here. Legal: {}", legal.join(", "));
        return;
    }
    let next = state.apply(action).expect("action is legal");
    println!("  You  (P0) → {action}");
    if next.is_terminal() {
        session.state = Some(next.clone());
        session.finish(next);
        return;
    }
    session.state = Some(next);
    session.gto_respond();

    // After GTO responds, if it is P0's turn again (Check-Bet scenario), prompt.
    if let Some(state) = &session.state {
        if state.current_player() == Some(0) && !state.is_terminal() {
            let card = session.p0_card.unwrap();
            let legal: Vec<String> = state.legal_actions().iter().map(|a| a.to_string()).collect();
            println!(
                "  Your card [{card}] | history: {} | legal: {}",
                state.history(),
                legal.join(", ")
            );
        }
    }
}
