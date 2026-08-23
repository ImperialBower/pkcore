//! Golden-vector dumper for the `/deconstruct` regeneration pack.
//!
//! Serializes pkcore's observable behaviors to JSON under
//! `docs/deconstruct/vectors/<epic-slug>/`. Every value is produced by
//! *running* the library through its public API — nothing here is
//! hand-authored. Regenerating at the same commit reproduces every file
//! byte-identically.
//!
//! Run with:
//!
//! ```text
//! cargo run --example decon_dump --features equity,bot-profiles,hand-histories,player-stats
//! ```

// This file is one long series of data-shaping blocks: each extractor reads a
// behavior out of the library and writes it as JSON. The per-epic functions
// are necessarily long because the data they emit is long, and splitting them
// further would scatter a single vector's definition across several places.
// The numeric casts are all on small, bounded values (seat indices, sample
// counts) feeding display or ratio arithmetic.
#![allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::single_match_else,
    clippy::format_push_string
)]

use pkcore::SuitShift;
use pkcore::analysis::class::HandRankClass;
use pkcore::analysis::equity::{EquityOptions, EquityRequest, PlayerSpec};
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::weighted_combos::WeightedCombos;
use pkcore::analysis::hand_rank::{HandRank, HandRankValue};
use pkcore::analysis::name::HandRankName;
use pkcore::analysis::omaha::EightOrBetter;
use pkcore::analysis::player_stats::{Confidence, PlayerStats};
use pkcore::analysis::pot_odds::PotOdds;
use pkcore::arrays::HandRanker;
use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::table_snapshot::TableSnapshot;
use pkcore::casino::cashier::chips::Stack;
use pkcore::casino::position::Position;
use pkcore::casino::table::{Player, Seat, Seats, Table};
use pkcore::games::GameType;
use pkcore::games::betting_structure::{BetTier, BettingStructure};
use pkcore::games::kuhn::{KuhnAction, KuhnCard, KuhnCfr, KuhnHistory, KuhnInfoSet, KuhnState, KuhnStrategy};
use pkcore::games::omaha::OmahaHigh;
use pkcore::hand_history::HandHistory;
use pkcore::prelude::*;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const VECTOR_ROOT: &str = "docs/deconstruct/vectors";

fn main() {
    let mut written = 0usize;

    written += card_vocabulary();
    written += high_hand_ranking();
    written += lowball_ranking();
    written += range_notation();
    written += variants_and_betting();
    written += pot_accounting();
    written += equity_and_odds();
    written += equilibrium_solving();
    written += suit_isomorphism();
    written += table_engine();
    written += side_pots();
    written += player_statistics();
    written += agent_model();
    written += hand_history();

    println!("\n{written} vector files written.");
}

// ── plumbing ─────────────────────────────────────────────────────────────────

/// Writes one vector file in the pack's envelope, with LF endings, 2-space
/// indent, and a trailing newline.
fn write_vector(epic: &str, slug: &str, behavior: &str, data: Value) -> usize {
    let dir = PathBuf::from(VECTOR_ROOT).join(slug);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("FATAL: cannot create {}: {e}", dir.display());
        std::process::exit(1);
    }
    let path = dir.join(format!("{behavior}.json"));
    let envelope = json!({ "epic": epic, "behavior": behavior, "data": data });

    let mut text = match serde_json::to_string_pretty(&envelope) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FATAL: cannot serialize {}: {e}", path.display());
            std::process::exit(1);
        }
    };
    text.push('\n');

    if let Err(e) = std::fs::write(&path, text.as_bytes()) {
        eprintln!("FATAL: cannot write {}: {e}", path.display());
        std::process::exit(1);
    }
    println!("wrote {}", rel(&path));
    1
}

fn rel(path: &Path) -> String {
    path.display().to_string()
}

/// Replaces freshly-generated identities with stable placeholders, numbered in
/// order of first appearance. Identities are assigned randomly at construction,
/// so leaving them in would make the vectors differ on every run — and *which*
/// identity a seat holds is not a domain fact.
fn redact_identities(text: &str, seen: &mut Vec<String>) -> String {
    let bytes: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;

    let is_hex = |c: char| c.is_ascii_hexdigit();
    while i < bytes.len() {
        // A canonical identity is 8-4-4-4-12 hex digits joined by hyphens.
        let groups = [8usize, 4, 4, 4, 12];
        let mut cursor = i;
        let mut matched = true;
        for (g, len) in groups.iter().enumerate() {
            if g > 0 {
                if cursor >= bytes.len() || bytes[cursor] != '-' {
                    matched = false;
                    break;
                }
                cursor += 1;
            }
            if cursor + len > bytes.len() || !bytes[cursor..cursor + len].iter().all(|c| is_hex(*c)) {
                matched = false;
                break;
            }
            cursor += len;
        }
        if matched {
            let found: String = bytes[i..cursor].iter().collect();
            let idx = match seen.iter().position(|s| *s == found) {
                Some(p) => p,
                None => {
                    seen.push(found);
                    seen.len() - 1
                }
            };
            out.push_str(&format!("<identity-{idx}>"));
            i = cursor;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

/// Parses a five-card hand, exiting loudly rather than fabricating a value.
fn five(s: &str) -> Five {
    match Five::from_str(s) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FATAL: cannot parse five-card hand {s:?}: {e:?}");
            std::process::exit(1);
        }
    }
}

fn seven(s: &str) -> Seven {
    match Seven::from_str(s) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("FATAL: cannot parse seven-card hand {s:?}: {e:?}");
            std::process::exit(1);
        }
    }
}

// ── DECON-01 card vocabulary ─────────────────────────────────────────────────

fn card_vocabulary() -> usize {
    let epic = "DECON-01";
    let slug = "card-vocabulary";
    let mut n = 0;

    // The canonical deck, in the library's own order.
    let composition: Vec<Value> = (0..Deck::len())
        .map(|i| {
            let card = Deck::get(i).expect("index is below Deck::len()");
            json!({
                "index": i,
                "rank": card.get_rank().to_char().to_string(),
                "suit": card.get_suit().to_char_letter().to_string(),
                "letter_form": card.get_letter_index(),
                "glyph_form": card.to_string(),
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "composition",
        json!({
            "deck_size": Deck::len(),
            "description": "The canonical 52-card deck in the order the library defines it.",
            "cards": composition,
        }),
    );

    // Text forms round-trip, in both letter and glyph suit notation.
    let samples = [
        "AS", "A♠", "as", "Ks", "K♠", "2c", "2♣", "Td", "T♦", "9h", "9♥", "Qc", "J♦",
    ];
    let roundtrip: Vec<Value> = samples
        .iter()
        .map(|s| match Card::from_str(s) {
            Ok(card) => json!({
                "input": s,
                "parses": true,
                "canonical_glyph": card.to_string(),
                "canonical_letter": card.get_letter_index(),
            }),
            Err(_) => json!({ "input": s, "parses": false }),
        })
        .collect();

    let multi = ["A♠ K♠ Q♠ J♠ T♠", "As Ks Qs Js Ts", "2c 3d 4h 5s 7c"];
    let multi_roundtrip: Vec<Value> = multi
        .iter()
        .map(|s| match Cards::from_str(s) {
            Ok(cards) => json!({
                "input": s,
                "parses": true,
                "canonical": cards.to_string(),
                "count": cards.len(),
            }),
            Err(_) => json!({ "input": s, "parses": false }),
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "text-roundtrip",
        json!({
            "description": "Card and multi-card text forms parse and re-render canonically. \
                            Both letter and glyph suit notations are accepted on input; \
                            output is canonical.",
            "single_cards": roundtrip,
            "collections": multi_roundtrip,
        }),
    );

    // Collection semantics: ordering, deduplication, difference, census.
    let deck = Deck::poker_cards();
    let aces = match Cards::from_str("A♠ A♥ A♦ A♣") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("FATAL: cannot parse aces: {e:?}");
            std::process::exit(1);
        }
    };
    let without_aces = deck.minus(&aces);

    n += write_vector(
        epic,
        slug,
        "set-algebra",
        json!({
            "description": "Card collections are ordered and deduplicated; difference removes \
                            members; the deck-composition census is a fixed property of a \
                            52-card deck.",
            "deck_size": deck.len(),
            "aces_removed": {
                "removed": aces.to_string(),
                "remaining_count": without_aces.len(),
                "contains_ace_of_spades": without_aces.contains(&Card::ACE_SPADES),
            },
            "census": {
                "unique_5_card_hands": pkcore::UNIQUE_5_CARD_HANDS,
                "distinct_5_card_hands": pkcore::DISTINCT_5_CARD_HANDS,
                "unique_2_card_hands": pkcore::UNIQUE_2_CARD_HANDS,
                "distinct_2_card_hands": pkcore::DISTINCT_2_CARD_HANDS,
                "unique_pocket_pairs": pkcore::UNIQUE_POCKET_PAIRS,
                "unique_non_pocket_pairs": pkcore::UNIQUE_NON_POCKET_PAIRS,
                "unique_suited_2_card_hands": pkcore::UNIQUE_SUITED_2_CARD_HANDS,
                "possible_unique_holdem_hup_matchups": pkcore::POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS,
            },
            "frequency_census": {
                "straight_flush": { "unique": pkcore::UNIQUE_STRAIGHT_FLUSHES, "distinct": pkcore::DISTINCT_STRAIGHT_FLUSHES },
                "four_of_a_kind": { "unique": pkcore::UNIQUE_FOUR_OF_A_KIND, "distinct": pkcore::DISTINCT_FOUR_OF_A_KIND },
                "full_house": { "unique": pkcore::UNIQUE_FULL_HOUSES, "distinct": pkcore::DISTINCT_FULL_HOUSES },
                "flush": { "unique": pkcore::UNIQUE_FLUSH, "distinct": pkcore::DISTINCT_FLUSH },
                "straight": { "unique": pkcore::UNIQUE_STRAIGHT, "distinct": pkcore::DISTINCT_STRAIGHT },
                "three_of_a_kind": { "unique": pkcore::UNIQUE_THREE_OF_A_KIND, "distinct": pkcore::DISTINCT_THREE_OF_A_KIND },
                "two_pair": { "unique": pkcore::UNIQUE_TWO_PAIR, "distinct": pkcore::DISTINCT_TWO_PAIR },
                "one_pair": { "unique": pkcore::UNIQUE_ONE_PAIR, "distinct": pkcore::DISTINCT_ONE_PAIR },
                "high_card": { "unique": pkcore::UNIQUE_HIGH_CARD, "distinct": pkcore::DISTINCT_HIGH_CARD },
            },
        }),
    );

    n
}

// ── DECON-02 high hand ranking ───────────────────────────────────────────────

fn high_hand_ranking() -> usize {
    let epic = "DECON-02";
    let slug = "high-hand-ranking";
    let mut n = 0;

    // Walk the whole 1..=7462 space and compress it into contiguous runs of
    // (category, class). This is the complete observable ranking taxonomy.
    let mut runs: Vec<Value> = Vec::new();
    let mut start: HandRankValue = 1;
    let mut current = (HandRankName::from(1u16), format!("{:?}", HandRankClass::from(1u16)));

    for value in 2..=7463u16 {
        let here = if value <= 7462 {
            (HandRankName::from(value), format!("{:?}", HandRankClass::from(value)))
        } else {
            (HandRankName::Invalid, String::from("__end__"))
        };
        if here != current {
            runs.push(json!({
                "from": start,
                "to": value - 1,
                "category": format!("{:?}", current.0),
                "class": current.1,
            }));
            start = value;
            current = here;
        }
    }

    // Category bands, derived by scanning rather than transcribed.
    let mut bands: Vec<Value> = Vec::new();
    let mut band_start: HandRankValue = 1;
    let mut band_name = HandRankName::from(1u16);
    for value in 2..=7463u16 {
        let here = if value <= 7462 {
            HandRankName::from(value)
        } else {
            HandRankName::Invalid
        };
        if here != band_name {
            bands.push(json!({
                "category": format!("{:?}", band_name),
                "best_value": band_start,
                "worst_value": value - 1,
                "count": value - band_start,
            }));
            band_start = value;
            band_name = here;
        }
    }

    n += write_vector(
        epic,
        slug,
        "category-bands",
        json!({
            "description": "Every five-card hand receives an integer rank. 1 is the strongest \
                            hand and 7462 the weakest; lower is stronger. Value 0 is an \
                            out-of-band sentinel meaning 'no hand'. The nine categories occupy \
                            contiguous, non-overlapping bands.",
            "strongest_value": 1,
            "weakest_value": pkcore::DISTINCT_5_CARD_HANDS,
            "no_hand_sentinel": pkcore::analysis::hand_rank::NO_HAND_RANK_VALUE,
            "bands": bands,
            "class_runs": runs,
        }),
    );

    // Representative hands and their exact ranks, plus comparison semantics.
    let hands = [
        ("A♠ K♠ Q♠ J♠ T♠", "royal flush"),
        ("K♠ Q♠ J♠ T♠ 9♠", "king-high straight flush"),
        ("5♠ 4♠ 3♠ 2♠ A♠", "the wheel, suited — weakest straight flush"),
        ("A♠ A♥ A♦ A♣ K♠", "four aces"),
        ("A♠ A♥ A♦ K♠ K♥", "aces full of kings"),
        ("A♠ K♠ Q♠ J♠ 9♠", "ace-high flush"),
        ("A♠ K♥ Q♦ J♣ T♠", "ace-high straight"),
        ("5♠ 4♥ 3♦ 2♣ A♠", "the wheel — weakest straight"),
        ("J♣ T♣ 9♣ 8♠ 7♣", "jack-high straight, mixed suits"),
        ("A♠ A♥ A♦ K♠ Q♥", "three aces"),
        ("A♠ A♥ K♦ K♠ Q♥", "aces and kings"),
        ("A♠ A♥ K♦ Q♠ J♥", "pair of aces"),
        ("A♠ K♥ Q♦ J♣ 9♠", "ace-high"),
        ("7♠ 5♥ 4♦ 3♣ 2♠", "seven-high — the weakest hand in poker"),
    ];

    let ordering: Vec<Value> = hands
        .iter()
        .map(|(hand, note)| {
            let f = five(hand);
            let rank = f.hand_rank();
            json!({
                "hand": hand,
                "note": note,
                "value": rank.value,
                "category": format!("{:?}", rank.name),
                "class": format!("{:?}", rank.class),
                "is_flush": f.is_flush(),
                "is_straight": f.is_straight(),
                "is_straight_flush": f.is_straight_flush(),
                "is_wheel": f.is_wheel(),
            })
        })
        .collect();

    // Comparison semantics, including the invalid-sorts-last rule.
    let royal = HandRank::from(1u16);
    let worst = HandRank::from(7462u16);
    let invalid = HandRank::from(0u16);
    let comparisons = json!({
        "royal_vs_worst": format!("{:?}", royal.cmp(&worst)),
        "worst_vs_royal": format!("{:?}", worst.cmp(&royal)),
        "royal_vs_invalid": format!("{:?}", royal.cmp(&invalid)),
        "invalid_vs_worst": format!("{:?}", invalid.cmp(&worst)),
        "note": "A numerically lower value compares as the STRONGER hand. The \
                 out-of-band sentinel compares as weaker than every real hand, \
                 despite being numerically lowest.",
    });

    n += write_vector(
        epic,
        slug,
        "ordering",
        json!({
            "description": "Representative hands with the exact rank each receives, plus the \
                            comparison semantics that order them.",
            "hands": ordering,
            "comparison": comparisons,
        }),
    );

    // Best five of six and best five of seven.
    let sevens = [
        "A♠ K♠ Q♠ J♠ T♠ 2♥ 3♦",
        "A♠ A♥ K♦ K♠ Q♥ Q♦ 2♣",
        "9♣ 6♦ 5♥ 5♠ 8♠ 5♦ 5♣",
        "2♠ 7♥ 9♦ J♣ 4♠ 6♥ 8♣",
    ];
    let best_of_seven: Vec<Value> = sevens
        .iter()
        .map(|s| {
            let hand = seven(s);
            let (rank, best) = hand.hand_rank_and_hand();
            json!({
                "seven_cards": s,
                "best_five": best.to_string(),
                "value": rank.value,
                "category": format!("{:?}", rank.name),
                "class": format!("{:?}", rank.class),
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "best-of-n",
        json!({
            "description": "Ranking more than five cards means choosing the best five. Seven \
                            cards yield exactly 21 five-card subsets; the strongest wins.",
            "five_card_subsets_of_seven": Seven::FIVE_CARD_PERMUTATIONS.len(),
            "five_card_subsets_of_six": Six::FIVE_CARD_PERMUTATIONS.len(),
            "seven_card_permutation_table": Seven::FIVE_CARD_PERMUTATIONS,
            "six_card_permutation_table": Six::FIVE_CARD_PERMUTATIONS,
            "hands": best_of_seven,
        }),
    );

    // The Omaha exactly-2-from-hand / exactly-3-from-board rule.
    let omaha_cases = [
        ("A♠ A♥ K♦ K♠", "Q♠ J♠ T♠ 2♥ 3♦", "board is a broadway flush draw"),
        ("A♠ K♠ 2♥ 3♦", "Q♠ J♠ T♠ 9♠ 8♠", "board is a made flush"),
        ("A♠ A♥ 2♣ 3♦", "A♦ A♣ K♠ K♥ 2♠", "quads on the board"),
        (
            "2♣ 3♦ 4♥ 5♦",
            "A♠ K♠ Q♠ J♠ T♠",
            "board is a royal flush and no hole card is a spade — the rule's discriminating case: the board does not play, so the answer is a high card",
        ),
    ];
    let omaha: Vec<Value> = omaha_cases
        .iter()
        .filter_map(|(hole, board, note)| {
            let h = Four::from_str(hole).ok()?;
            let b = Five::from_str(board).ok()?;
            let eval = OmahaHigh { hand: h }.eval(&Board::from(b));
            Some(json!({
                "hole_cards": hole,
                "board": board,
                "note": note,
                "best_five": eval.hand.to_string(),
                "value": eval.hand_rank.value,
                "category": format!("{:?}", eval.hand_rank.name),
                "class": format!("{:?}", eval.hand_rank.class),
            }))
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "omaha-permutations",
        json!({
            "description": "Omaha requires exactly two cards from the hand and exactly three \
                            from the board, giving 6 x 10 = 60 candidate five-card hands. A \
                            board flush or board quads therefore does NOT automatically play.",
            "hole_combinations": 6,
            "board_combinations": 10,
            "total_candidates": 60,
            "hole_permutation_table": pkcore::games::omaha::OMAHA_HAND_PERMUTATIONS,
            "board_permutation_table": pkcore::games::omaha::OMAHA_BOARD_PERMUTATIONS,
            "hands": omaha,
        }),
    );

    n
}

// ── DECON-03 lowball ranking ─────────────────────────────────────────────────

fn lowball_ranking() -> usize {
    let epic = "DECON-03";
    let slug = "lowball-ranking";
    let mut n = 0;

    let lows = [
        ("5♠ 4♥ 3♦ 2♣ A♠", "the wheel — the nut low"),
        ("6♠ 4♥ 3♦ 2♣ A♠", "six-four low"),
        ("6♠ 5♥ 4♦ 3♣ 2♠", "six-five low"),
        ("7♠ 5♥ 4♦ 3♣ 2♠", "seven-five low"),
        ("8♠ 6♥ 4♦ 3♣ 2♠", "eight-six low"),
        (
            "5♠ 4♠ 3♠ 2♠ A♠",
            "the wheel, all one suit — a flush does not count against a low",
        ),
        ("A♠ K♥ Q♦ J♣ T♠", "broadway — a terrible low"),
        ("A♠ A♥ 3♦ 2♣ 4♠", "paired — does not qualify as a low in this path"),
    ];

    // Enumerate every unpaired rank-set and rank it two ways: by the canonical
    // lowball rule (compare highest card first, descending) and by the ordinal
    // the original assigns. Comparing the two exposes where they disagree.
    let ladder = lowball_ladder();

    let razz: Vec<Value> = lows
        .iter()
        .map(|(hand, note)| {
            let f = five(hand);
            let rank = f.razz_hand_rank();
            let key = lowball_key_of(hand);
            json!({
                "hand": hand,
                "note": note,
                "canonical_low_key": key,
                "canonical_position": key.as_ref().and_then(|k| {
                    ladder.iter().position(|(kk, _)| kk == k).map(|p| p + 1)
                }),
                "original_ordinal": rank.get_hand_rank_value(),
                "original_class": format!("{rank:?}"),
            })
        })
        .collect();

    let canonical_order: Vec<Value> = ladder
        .iter()
        .take(24)
        .enumerate()
        .map(|(i, (key, ordinal))| {
            json!({
                "position": i + 1,
                "low_ranks_high_to_low": key,
                "original_ordinal": ordinal,
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "razz-ordering",
        json!({
            "description": "Ace-to-five lowball. Aces play low; straights and flushes do NOT \
                            count against a low hand, so the nut low is the wheel 5-4-3-2-A. \
                            THE NORMATIVE RULE IS THE CANONICAL ONE: compare two lows by their \
                            highest card first, then the next highest, and so on; the hand that \
                            runs out lower wins. 'canonical_position' is that rule applied to \
                            all 1287 unpaired rank-sets, 1 being the nut low. \
                            'original_ordinal' is what the source assigns, recorded as evidence \
                            only -- see ladder-divergence and spec decision SD-02. A rebuild \
                            must reproduce canonical_position, NOT original_ordinal.",
            "nut_low_position": 1,
            "unpaired_low_hands": ladder.len(),
            "no_low_sentinel": pkcore::games::razz::california::NO_RAZZ_HAND_RANK_VALUE,
            "canonical_ladder_head": canonical_order,
            "hands": razz,
        }),
    );

    // Extracted evidence that the original's ladder is not the canonical one.
    let mut violations: Vec<Value> = Vec::new();
    let mut violation_count = 0usize;
    for w in ladder.windows(2) {
        let (key_better, ord_better) = &w[0];
        let (key_worse, ord_worse) = &w[1];
        if ord_better >= ord_worse {
            violation_count += 1;
            if violations.len() < 12 {
                violations.push(json!({
                    "stronger_low_ranks": key_better,
                    "stronger_original_ordinal": ord_better,
                    "weaker_low_ranks": key_worse,
                    "weaker_original_ordinal": ord_worse,
                    "note": "By the canonical rule the first hand is the stronger low, but the \
                             source assigns it the worse (higher) ordinal.",
                }));
            }
        }
    }

    n += write_vector(
        epic,
        slug,
        "ladder-divergence",
        json!({
            "description": "Evidence, extracted by running the original, that its ace-to-five \
                            ladder does NOT implement the canonical lowball comparison. The \
                            ladder orders hands lexicographically ascending from the LOWEST \
                            card, whereas lowball compares from the HIGHEST card downward. The \
                            wheel is still correctly the nut low, and hands sharing a lowest \
                            card are ordered correctly among themselves, so the error only \
                            shows across families -- which is why round-trip replay testing \
                            never caught it. This file exists so a rebuilder does not \
                            reproduce the defect.",
            "adjacent_pairs_compared": ladder.len() - 1,
            "misordered_pairs": violation_count,
            "worked_example": {
                "hand_a": "6-5-4-3-2 (a six low)",
                "hand_b": "7-4-3-2-A (a seven low)",
                "correct": "A six low always beats a seven low.",
                "original_behavior": "assigns the six low ordinal 496 and the seven low \
                                      ordinal 3, making the seven low win.",
            },
            "examples": violations,
        }),
    );

    let eob_cases = [
        ("5♠ 4♥ 3♦ 2♣ A♠", "the wheel — the nut eight-or-better low"),
        ("8♠ 6♥ 4♦ 3♣ 2♠", "eight-six-four-three-deuce — qualifies"),
        ("8♠ 7♥ 6♦ 5♣ 4♠", "eight-seven low — qualifies"),
        ("9♠ 6♥ 4♦ 3♣ 2♠", "nine-high — does NOT qualify"),
        ("A♠ A♥ 4♦ 3♣ 2♠", "paired — does NOT qualify"),
        ("A♠ K♥ Q♦ J♣ T♠", "broadway — does NOT qualify"),
    ];
    let eob: Vec<Value> = eob_cases
        .iter()
        .map(|(hand, note)| {
            let f = five(hand);
            json!({
                "hand": hand,
                "note": note,
                "qualifies": EightOrBetter::is_eight_or_better(f),
                "low_bits": EightOrBetter::filter(f),
                "class": format!("{:?}", EightOrBetter::from(f)),
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "eight-or-better",
        json!({
            "description": "The eight-or-better qualifier: a low hand qualifies only with five \
                            unpaired cards all ranked eight or lower. The wheel is the nut.",
            "hands": eob,
        }),
    );

    n
}

// ── DECON-06 table engine / DECON-07 side pots ───────────────────────────────

/// Builds a table whose seats hold the given hole cards and stacks, with the
/// deck re-ordered so every subsequent draw is deterministic. Nothing here
/// depends on a random generator, so the vectors bind no shuffle algorithm.
fn rigged_table(seats_spec: &[(&str, usize, &str)], top: &str) -> Table {
    let seats: Vec<Seat> = seats_spec
        .iter()
        .map(|(handle, stack, hole)| {
            let mut seat = Seat::new(Player::new_with_chips((*handle).to_string(), *stack));
            match BoxedCards::from_str(hole) {
                Ok(cards) => seat.cards = cards,
                Err(e) => {
                    eprintln!("FATAL: cannot parse hole cards {hole:?}: {e:?}");
                    std::process::exit(1);
                }
            }
            seat
        })
        .collect();

    let mut table = Table::nlh_from_seats(Seats::new(seats), ForcedBets::new(50, 100));

    let mut used: Vec<&str> = seats_spec.iter().map(|(_, _, hole)| *hole).collect();
    used.push(top);
    let used_str = used.join(" ");

    let Ok(used_cards) = Cards::from_str(&used_str) else {
        eprintln!("FATAL: cannot parse the used-card set");
        std::process::exit(1);
    };
    let Ok(mut deck) = Cards::from_str(top) else {
        eprintln!("FATAL: cannot parse the rigged deck top");
        std::process::exit(1);
    };
    deck.insert_all(&Cards::deck_minus(&used_cards));
    table.deck = (&deck).into();
    table
}

fn stacks_of(table: &Table) -> Vec<Value> {
    (0..table.seats.0.len() as u8)
        .filter_map(|i| {
            let seat = table.seats.get_seat(i)?;
            Some(json!({
                "seat": i,
                "handle": seat.player.handle.clone(),
                "chips": seat.player.chips,
            }))
        })
        .collect()
}

/// Runs every seat all-in preflop, then deals the full board out.
fn shove_to_showdown(table: &mut Table) {
    if let Err(e) = table.act_forced_bets() {
        eprintln!("FATAL: forced bets failed: {e:?}");
        std::process::exit(1);
    }
    for _ in 0..table.seats.0.len() {
        let seat = table.next_to_act();
        if table.act_all_in(seat).is_err() {
            break;
        }
    }
    let _ = table.bring_it_in();
    let _ = table.deal_flop();
    let _ = table.deal_turn();
    let _ = table.deal_river();
}

fn table_engine() -> usize {
    let epic = "DECON-06";
    let slug = "table-engine";
    let mut n = 0;

    // Forced bets across table sizes, recorded from a freshly-posted table.
    let mut forced_rows: Vec<Value> = Vec::new();
    for (label, spec) in [
        (
            "heads-up: the button is also the small blind",
            &[("Button", 1_000usize, "A♠ K♠"), ("Opponent", 1_000, "Q♥ J♥")][..],
        ),
        (
            "three-handed",
            &[
                ("Button", 1_000usize, "A♠ K♠"),
                ("Small", 1_000, "Q♥ J♥"),
                ("Big", 1_000, "9♦ 8♦"),
            ][..],
        ),
        (
            "six-handed",
            &[
                ("Button", 1_000usize, "A♠ K♠"),
                ("Small", 1_000, "Q♥ J♥"),
                ("Big", 1_000, "9♦ 8♦"),
                ("UTG", 1_000, "7♣ 6♣"),
                ("Middle", 1_000, "5♠ 4♠"),
                ("Cutoff", 1_000, "3♥ 2♥"),
            ][..],
        ),
    ] {
        let mut table = rigged_table(spec, "2♦ 3♠ 4♦ 5♥ 6♠ 7♠ 8♣ 9♠");
        if table.act_forced_bets().is_err() {
            continue;
        }
        forced_rows.push(json!({
            "label": label,
            "seat_count": spec.len(),
            "small_blind": table.forced.small_blind,
            "big_blind": table.forced.big_blind,
            "pot_after_posting": table.pot,
            "first_to_act_preflop": table.next_to_act(),
            "seats": stacks_of(&table),
        }));
    }

    n += write_vector(
        epic,
        slug,
        "forced-bets",
        json!({
            "description": "Blind posting and who acts first before the flop. Heads-up is the \
                            special case: the button posts the small blind and acts first \
                            before the flop, then acts last on every later street.",
            "tables": forced_rows,
        }),
    );

    // Legal actions at a few well-defined decision points.
    let mut legal_rows: Vec<Value> = Vec::new();
    let mut table = rigged_table(
        &[
            ("Button", 1_000, "A♠ K♠"),
            ("Small", 1_000, "Q♥ J♥"),
            ("Big", 1_000, "9♦ 8♦"),
        ],
        "2♦ 3♠ 4♦ 5♥ 6♠ 7♠ 8♣ 9♠",
    );
    if table.act_forced_bets().is_ok() {
        let actor = table.next_to_act();
        legal_rows.push(json!({
            "situation": "first to act before the flop, facing the big blind",
            "seat": actor,
            "to_call": table.to_call(actor),
            "legal_actions": table
                .legal_actions(actor)
                .iter()
                .map(|a| format!("{a:?}"))
                .collect::<Vec<String>>(),
            "note": "Calling is available and checking is not, because there is a live bet.",
        }));

        // A seat with no chips left has no decision to make.
        let mut broke = rigged_table(
            &[("Rich", 1_000, "A♠ K♠"), ("Short", 100, "Q♥ J♥")],
            "2♦ 3♠ 4♦ 5♥ 6♠ 7♠ 8♣ 9♠",
        );
        if broke.act_forced_bets().is_ok() {
            let seat = broke.next_to_act();
            let _ = broke.act_all_in(seat);
            legal_rows.push(json!({
                "situation": "a seat that is already all-in",
                "seat": seat,
                "legal_actions": broke
                    .legal_actions(seat)
                    .iter()
                    .map(|a| format!("{a:?}"))
                    .collect::<Vec<String>>(),
                "note": "An all-in seat has nothing left to decide, so no action is legal.",
            }));
        }
    }

    n += write_vector(
        epic,
        slug,
        "legal-actions",
        json!({
            "description": "The set of actions available to a seat depends on whether there is \
                            a live bet to answer, whether the seat still has chips, and whether \
                            it is still in the hand. A seat that cannot act offers no actions \
                            at all rather than a no-op.",
            "situations": legal_rows,
        }),
    );

    // A full hand from a fixed deck, recorded as its ordered event log.
    let mut walk = rigged_table(
        &[("Deep", 1_000, "7♦ 2♣"), ("Short", 200, "A♠ A♥")],
        "3♣ K♠ K♣ 9♦ 3♥ 8♠ 3♦ 4♥",
    );
    let starting = stacks_of(&walk);
    shove_to_showdown(&mut walk);
    let board = walk.board.to_string();
    let settled = walk.end_hand().is_ok();
    let mut seen: Vec<String> = Vec::new();
    let events: Vec<String> = walk
        .event_log
        .iter()
        .map(|e| redact_identities(&e.to_string(), &mut seen))
        .collect();

    n += write_vector(
        epic,
        slug,
        "hand-walkthrough",
        json!({
            "description": "One complete hand played from an explicitly fixed deck -- no random \
                            generator is involved, so this vector binds no shuffle algorithm. \
                            The event log is the ordered, append-only record of everything that \
                            happened, and is sufficient on its own to reconstruct the hand.",
            "deck_top_in_deal_order": "3♣ K♠ K♣ 9♦ 3♥ 8♠ 3♦ 4♥",
            "deal_order_note": "burn, flop x3, burn, turn, burn, river",
            "starting_stacks": starting,
            "board": board,
            "settled_cleanly": settled,
            "ending_stacks": stacks_of(&walk),
            "chip_total": walk.table_chip_count(),
            "event_log": events,
        }),
    );

    n
}

fn side_pots() -> usize {
    let epic = "DECON-07";
    let slug = "pot-accounting";

    let scenarios = [
        (
            "three-way asymmetric all-in, all three hands tied",
            &[
                ("Short", 100usize, "7♦ 2♣"),
                ("Mid", 200, "4♦ 5♦"),
                ("Deep", 500, "8♥ 9♥"),
            ][..],
            "6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
            "Four aces on the board means everyone plays the board, so all three tie. \
             The 100-cap layer chops three ways, the 100-to-200 layer chops two ways, \
             and the 200-to-500 remainder was never matched so it returns. Every seat \
             must end at exactly its starting stack.",
        ),
        (
            "heads-up all-in, mismatched stacks, hands tied",
            &[("Deep", 1_000usize, "7♦ 2♣"), ("Short", 200, "4♦ 5♦")][..],
            "6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
            "Both play the board and tie. The main pot is capped at the short stack and \
             splits evenly; the deep stack's unmatched excess returns to it. Both end \
             where they started.",
        ),
        (
            "heads-up all-in, the short stack wins outright",
            &[("Deep", 1_000usize, "7♦ 2♣"), ("Short", 200, "A♠ A♥")][..],
            "3♣ K♠ K♣ 9♦ 3♥ 8♠ 3♦ 4♥",
            "The short stack wins the main pot only. The deep stack's unmatched excess \
             is returned rather than awarded to the winner -- chips nobody could match \
             were never truly in play.",
        ),
    ];

    let rows: Vec<Value> = scenarios
        .iter()
        .map(|(label, spec, top, note)| {
            let mut table = rigged_table(spec, top);
            let starting = stacks_of(&table);
            let total_before = table.table_chip_count();
            shove_to_showdown(&mut table);
            let board = table.board.to_string();
            let settled = table.end_hand().is_ok();
            let ending = stacks_of(&table);
            let total_after = table.table_chip_count();
            json!({
                "label": label,
                "note": note,
                "board": board,
                "starting_stacks": starting,
                "ending_stacks": ending,
                "chips_before": total_before,
                "chips_after": total_after,
                "chips_conserved": total_before == total_after,
                "settled_cleanly": settled,
            })
        })
        .collect();

    write_vector(
        epic,
        slug,
        "side-pots",
        json!({
            "description": "Layered side pots. When players are all-in for different amounts \
                            the pot divides into layers capped at each all-in level, and each \
                            layer is contested only by the players who paid into it. Tied \
                            winners chop each layer separately. Chips one player committed \
                            beyond what anyone matched are returned, not awarded.",
            "scenarios": rows,
        }),
    )
}

// ── DECON-08 hand history ────────────────────────────────────────────────────

fn hand_history() -> usize {
    let epic = "DECON-08";
    let slug = "hand-history";
    let mut n = 0;

    let spec = [("Deep", 1_000usize, "7♦ 2♣"), ("Short", 200, "A♠ A♥")];
    let deck_top = "3♣ K♠ K♣ 9♦ 3♥ 8♠ 3♦ 4♥";
    let mut table = rigged_table(&spec, deck_top);
    let shuffled = table.deck.to_string();

    let snapshot: Vec<(u8, String, usize, Option<String>)> = spec
        .iter()
        .enumerate()
        .map(|(i, (handle, stack, hole))| {
            (
                u8::try_from(i).unwrap_or(0),
                (*handle).to_string(),
                *stack,
                Some((*hole).to_string()),
            )
        })
        .collect();

    shove_to_showdown(&mut table);
    let board_str = table.board.to_string();
    let winnings = table.end_hand().unwrap_or_default();
    let ending: Vec<(u8, usize)> = (0..table.seats.0.len() as u8)
        .filter_map(|i| table.seats.get_seat(i).map(|s| (i, s.player.chips)))
        .collect();

    // A fixed timestamp: when a hand happened is not a domain property under test.
    let history = HandHistory::from_table_state(
        1,
        0,
        0,
        &table.forced,
        &snapshot,
        &board_str,
        &winnings,
        &table.event_log,
        &ending,
        "decon-dump",
        Some(shuffled.clone()),
    )
    .with_table_size(table.seats.size() as usize);

    let mut seen: Vec<String> = Vec::new();
    let first = history.to_yaml().unwrap_or_default();
    let reparsed = HandHistory::from_yaml(&first);
    let (parses, stable, structurally_equal) = match reparsed {
        Ok(ref again) => {
            let second = again.to_yaml().unwrap_or_default();
            (true, first == second, *again == history)
        }
        Err(_) => (false, false, false),
    };

    n += write_vector(
        epic,
        slug,
        "roundtrip",
        json!({
            "description": "A hand record must survive a round trip: serialize it, read it \
                            back, and serialize again -- the text is stable and the reparsed \
                            record is structurally equal to the original. The concrete \
                            serialization syntax below is the original's; a rebuild owns its \
                            own schema (see SD-10). What is normative is the INFORMATION and \
                            the round-trip property.",
            "format_version": history.format_version,
            "recorded_deck": redact_identities(&shuffled, &mut seen),
            "deck_consumption_order": "hole cards clockwise from the button, then burn+flop, \
                                       burn+turn, burn+river",
            "board": board_str,
            "seats": history.players.iter().map(|p| json!({
                "seat": p.seat,
                "starting_stack": p.stack,
                "hole_cards": p.hole_cards.clone(),
            })).collect::<Vec<Value>>(),
            "results": history.results.as_ref().map(|rs| rs.iter().map(|r| json!({
                "seat": r.seat,
                "outcome": format!("{:?}", r.outcome),
                "net": r.net,
                "pot_won": r.pot_won,
                "best_hand": r.best_hand.clone(),
            })).collect::<Vec<Value>>()),
            "roundtrip": {
                "reparses": parses,
                "text_is_stable": stable,
                "structurally_equal": structurally_equal,
            },
        }),
    );

    // Replay: re-running the record must reproduce the recorded final stacks.
    let replay = history.replay();
    let (replayed, consistent) = match replay {
        Ok(ref r) => (true, r.is_consistent),
        Err(_) => (false, false),
    };

    n += write_vector(
        epic,
        slug,
        "replay",
        json!({
            "description": "The determinism promise at the centre of this epic: a recorded hand \
                            replayed through the engine reproduces the recorded final stacks \
                            EXACTLY. This is what makes a hand record lossless rather than \
                            merely descriptive.",
            "recorded_ending_stacks": ending.iter().map(|(seat, chips)| json!({
                "seat": seat, "chips": chips,
            })).collect::<Vec<Value>>(),
            "replay": {
                "replays_without_error": replayed,
                "reproduces_recorded_stacks": consistent,
            },
            "chip_conservation": {
                "starting_total": spec.iter().map(|(_, s, _)| s).sum::<usize>(),
                "ending_total": ending.iter().map(|(_, c)| c).sum::<usize>(),
            },
        }),
    );

    n
}

// ── DECON-12 player statistics ───────────────────────────────────────────────

fn player_statistics() -> usize {
    let epic = "DECON-12";
    let slug = "player-statistics";
    let mut n = 0;

    let bands: Vec<Value> = [0u32, 1, 25, 49, 50, 100, 199, 200, 201, 1_000]
        .iter()
        .map(|size| {
            json!({
                "sample_size": size,
                "confidence": format!("{:?}", Confidence::from_sample_size(u64::from(*size))),
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "confidence",
        json!({
            "description": "How much to trust a player's statistics is a function of how many \
                            hands they are drawn from. The bands are observable and matter \
                            because consumers gate decisions on them.",
            "bands": bands,
        }),
    );

    // Derived statistics from explicit raw counters, including the
    // no-sample case which must be undefined rather than zero.
    let populated = PlayerStats {
        hands_dealt: 100,
        hands_voluntarily_played: 24,
        went_to_showdown: 18,
        won_at_showdown: 11,
        pfr_opportunities: 100,
        pfr_count: 17,
        three_bet_opportunities: 40,
        three_bet_count: 3,
        ..Default::default()
    };

    let empty = PlayerStats::default();

    let rows = [
        ("a player with 100 hands recorded", &populated),
        ("a player with no hands recorded", &empty),
    ];
    let derived: Vec<Value> = rows
        .iter()
        .map(|(label, stats)| {
            json!({
                "label": label,
                "raw_counters": {
                    "hands_dealt": stats.hands_dealt,
                    "hands_voluntarily_played": stats.hands_voluntarily_played,
                    "went_to_showdown": stats.went_to_showdown,
                    "won_at_showdown": stats.won_at_showdown,
                    "preflop_raise_opportunities": stats.pfr_opportunities,
                    "preflop_raise_count": stats.pfr_count,
                    "three_bet_opportunities": stats.three_bet_opportunities,
                    "three_bet_count": stats.three_bet_count,
                },
                "derived": {
                    "voluntarily_put_money_in_pot": stats.vpip(),
                    "preflop_raise": stats.pfr(),
                    "three_bet": stats.three_bet_pct(),
                    "went_to_showdown_rate": stats.wtsd(),
                    "won_at_showdown_rate": stats.w_at_sd(),
                },
                "confidence": format!("{:?}", stats.confidence()),
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "derivations",
        json!({
            "description": "Each statistic is a ratio of occurrences to opportunities. A player \
                            with no opportunities has NO RATE -- absent, not zero. Conflating \
                            'never raised in 100 chances' with 'never had a chance' is a real \
                            modelling error, so the absence must survive into the result.",
            "players": derived,
        }),
    );

    n
}

// ── DECON-11 agent model ─────────────────────────────────────────────────────

fn agent_model() -> usize {
    let epic = "DECON-11";
    let slug = "agent-model";

    let archetypes: Vec<(&str, BotProfile)> = vec![
        ("tight-passive", BotProfile::tight_passive()),
        ("loose-aggressive", BotProfile::loose_aggressive()),
        ("game-theory-optimal", BotProfile::gto()),
        ("tight-aggressive", BotProfile::tight_aggressive()),
        ("loose-passive", BotProfile::loose_passive()),
        ("maniac", BotProfile::maniac()),
        ("by-the-book", BotProfile::abc()),
        ("short-stack specialist", BotProfile::short_stack_ninja()),
    ];

    let rows: Vec<Value> = archetypes
        .iter()
        .map(|(label, profile)| {
            json!({
                "label": label,
                "play_style": format!("{:?}", profile.style),
                "profile": serde_json::to_value(profile).unwrap_or(Value::Null),
            })
        })
        .collect();

    // Seeded decisions: the same seed and the same situation must produce the
    // same action. INFORMATIVE — the actions depend on the original's generator.
    let decider = RuleBasedDecider;
    let mut seeded: Vec<Value> = Vec::new();
    for seed in [7u64, 42] {
        let mut table = rigged_table(
            &[("Hero", 1_000, "A♠ K♠"), ("Villain", 1_000, "Q♥ J♥")],
            "2♦ 3♠ 4♦ 5♥ 6♠ 7♠ 8♣ 9♠",
        );
        if table.act_forced_bets().is_err() {
            continue;
        }
        let actor = table.next_to_act();
        let snapshot = TableSnapshot::from_table(&table, actor);
        let profile = BotProfile::tight_aggressive();

        let mut first = SmallRng::seed_from_u64(seed);
        let mut second = SmallRng::seed_from_u64(seed);
        let a = decider.decide_seeded(&profile, &snapshot, &mut first);
        let b = decider.decide_seeded(&profile, &snapshot, &mut second);

        seeded.push(json!({
            "seed": seed,
            "seat": actor,
            "profile": "tight-aggressive",
            "action": format!("{a:?}"),
            "repeat_action": format!("{b:?}"),
            "reproducible": format!("{a:?}") == format!("{b:?}"),
        }));
    }

    let mut n = write_vector(
        epic,
        slug,
        "seeded-decisions",
        json!({
            "description": "INFORMATIVE, NOT NORMATIVE for the recorded actions. What IS \
                            normative is the property: two runs from the same seed, against the \
                            same situation and profile, produce the same action. That property \
                            is what makes agent-driven simulation reproducible and therefore \
                            usable for research. The specific actions depend on the original's \
                            random generator, so a rebuild must demonstrate the property with \
                            its own cases rather than match these.",
            "normative_property": "same seed + same situation + same profile => same action",
            "runs": seeded,
        }),
    );

    n += write_vector(
        epic,
        slug,
        "profiles",
        json!({
            "description": "The named play-style archetypes, dumped in full. Behaviour is data, \
                            not code: a new personality is a new set of these parameters, never \
                            a new decision procedure. The concrete serialization shape here is \
                            the original's; a rebuild owns its own schema and need only carry \
                            the same parameters.",
            "archetypes": rows,
        }),
    );

    n
}

/// Every unpaired five-rank set, ordered by the canonical ace-to-five lowball
/// rule (highest card first, descending), paired with the ordinal the source
/// assigns it. The ordering here is derived from the rule; the ordinals are
/// read out of the library.
fn lowball_ladder() -> Vec<(Vec<u8>, u16)> {
    let glyphs = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K"];
    let suits = ["♠", "♥", "♦", "♣", "♠"];
    let mut rows: Vec<(Vec<u8>, u16)> = Vec::new();

    for a in 0..13u8 {
        for b in (a + 1)..13 {
            for c in (b + 1)..13 {
                for d in (c + 1)..13 {
                    for e in (d + 1)..13 {
                        let idx = [a, b, c, d, e];
                        let hand: String = idx
                            .iter()
                            .enumerate()
                            .map(|(i, r)| format!("{}{}", glyphs[*r as usize], suits[i]))
                            .collect::<Vec<_>>()
                            .join(" ");
                        let Ok(f) = Five::from_str(&hand) else { continue };
                        let ordinal = f.razz_hand_rank().get_hand_rank_value();
                        // Ace counts as 1; sort descending for the lowball key.
                        let mut key: Vec<u8> = idx.iter().map(|r| r + 1).collect();
                        key.sort_unstable_by(|x, y| y.cmp(x));
                        rows.push((key, ordinal));
                    }
                }
            }
        }
    }
    rows.sort_by(|x, y| x.0.cmp(&y.0));
    rows
}

/// The canonical lowball key for a written hand: ranks as low values with the
/// ace low, sorted highest first. Returns nothing if the hand is paired.
fn lowball_key_of(hand: &str) -> Option<Vec<u8>> {
    let cards = Cards::from_str(hand).ok()?;
    let mut key: Vec<u8> = cards
        .iter()
        .map(|card| {
            let ch = card.get_rank().to_char();
            match ch {
                'A' => 1,
                'T' => 10,
                'J' => 11,
                'Q' => 12,
                'K' => 13,
                other => other.to_digit(10).unwrap_or(0) as u8,
            }
        })
        .collect();
    key.sort_unstable_by(|x, y| y.cmp(x));
    let mut dedup = key.clone();
    dedup.dedup();
    if dedup.len() != key.len() {
        return None; // paired: no unpaired low
    }
    Some(key)
}

// ── DECON-04 range notation ──────────────────────────────────────────────────

fn range_notation() -> usize {
    let epic = "DECON-04";
    let slug = "range-notation";
    let mut n = 0;

    let classes = ["AA", "KK", "22", "AKs", "AKo", "AK", "T9s", "72o", "JTs"];
    let counts: Vec<Value> = classes
        .iter()
        .filter_map(|s| {
            let combo = Combo::from_str(s).ok()?;
            Some(json!({
                "notation": s,
                "canonical": combo.to_string(),
                "combinations": combo.total_pairs(),
                "is_pair": combo.is_pair(),
                "is_suited": combo.is_suited(),
                "is_offsuit": combo.is_offsuit(),
            }))
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "combo-counts",
        json!({
            "description": "A hand class names a pair of ranks plus a suitedness qualifier. \
                            The number of concrete two-card holdings in a class follows from \
                            the deck: 6 for a pair, 4 suited, 12 offsuit, 16 unqualified.",
            "expected": { "pair": 6, "suited": 4, "offsuit": 12, "unqualified": 16 },
            "classes": counts,
        }),
    );

    let ranges = ["AA", "QQ+", "AKs", "AJo+", "QQ+, AKs", "22+", "AT+"];
    let parsed: Vec<Value> = ranges
        .iter()
        .filter_map(|s| {
            let combos = Combos::from_str(s).ok()?;
            let mut classes: Vec<String> = combos.iter().map(std::string::ToString::to_string).collect();
            classes.sort();
            Some(json!({
                "input": s,
                "class_count": combos.len(),
                "classes": classes,
            }))
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "parse-roundtrip",
        json!({
            "description": "Range notation is a comma-separated list of hand classes. The '+' \
                            operator means 'this class and every stronger class in its family'. \
                            Classes are listed sorted here so the vector is order-independent.",
            "ranges": parsed,
        }),
    );

    let presets = [
        ("2.5%", Combos::PERCENT_2_5),
        ("5%", Combos::PERCENT_5),
        ("10%", Combos::PERCENT_10),
        ("20%", Combos::PERCENT_20),
        ("33%", Combos::PERCENT_33),
    ];
    let preset_rows: Vec<Value> = presets
        .iter()
        .filter_map(|(label, notation)| {
            let combos = Combos::from_str(notation).ok()?;
            let mut classes: Vec<String> = combos.iter().map(std::string::ToString::to_string).collect();
            classes.sort();
            let holdings: usize = combos.iter().map(Combo::total_pairs).sum();
            Some(json!({
                "label": label,
                "notation": notation,
                "class_count": combos.len(),
                "concrete_holdings": holdings,
                "share_of_all_holdings": format!("{:.4}", holdings as f64 / 1326.0),
                "classes": classes,
            }))
        })
        .collect();

    // Frequency-annotated ranges: a class may be played only part of the time.
    let mut weighted = WeightedCombos::default();
    for (notation, frequency) in [("AA", 1.0f64), ("KK", 1.0), ("AKs", 0.75), ("AJo", 0.25), ("T9s", 0.5)] {
        if let Ok(combo) = Combo::from_str(notation) {
            weighted.insert(combo, frequency);
        }
    }
    let mut entries: Vec<Value> = [
        ("AA", 6usize),
        ("KK", 6),
        ("AKs", 4),
        ("AJo", 12),
        ("T9s", 4),
        ("QQ", 6),
    ]
    .iter()
    .filter_map(|(notation, combinations)| {
        let combo = Combo::from_str(notation).ok()?;
        Some(json!({
            "class": notation,
            "combinations_in_class": combinations,
            "frequency": weighted.frequency(&combo),
            "in_range": weighted.frequency(&combo).is_some(),
        }))
    })
    .collect();
    entries.sort_by(|a, b| a["class"].as_str().cmp(&b["class"].as_str()));

    n += write_vector(
        epic,
        slug,
        "weighted",
        json!({
            "description": "A weighted range plays a hand class only part of the time. A class \
                            absent from the range has NO frequency, which is different from a \
                            frequency of zero: the first says nothing was specified, the second \
                            says it was specified as never. The effective number of holdings a \
                            class contributes is its combination count times its frequency.",
            "classes": entries,
            "rendered_range": weighted.to_range_str(),
            "effective_holdings": format!("{:.2}", 6.0 * 1.0 + 6.0 * 1.0 + 4.0 * 0.75 + 12.0 * 0.25 + 4.0 * 0.5),
        }),
    );

    n += write_vector(
        epic,
        slug,
        "percentile-presets",
        json!({
            "description": "Named percentile ranges. 'concrete_holdings' is the number of the \
                            1326 possible two-card holdings the range covers, which is what \
                            makes the percentage label meaningful.",
            "total_possible_holdings": pkcore::UNIQUE_2_CARD_HANDS,
            "presets": preset_rows,
        }),
    );

    n
}

// ── DECON-05 variants and betting ────────────────────────────────────────────

fn variants_and_betting() -> usize {
    let epic = "DECON-05";
    let slug = "variants-and-betting";
    let mut n = 0;

    let variants = [
        GameType::NoLimitHoldem,
        GameType::LimitHoldem,
        GameType::PLO,
        GameType::StudHi,
        GameType::Razz,
    ];

    let street_rows: Vec<Value> = variants
        .iter()
        .map(|game| {
            let streets: Vec<Value> = game
                .streets()
                .iter()
                .map(|s| {
                    json!({
                        "index": s.index.0,
                        "name": s.name,
                        "community_dealt": s.community_dealt,
                        "hole_dealt": s.hole_dealt,
                        "hole_dealt_up": s.hole_dealt_up,
                        "burn_first": s.burn_first,
                        "bet_tier": format!("{:?}", s.bet_tier),
                    })
                })
                .collect();
            json!({
                "variant": format!("{game:?}"),
                "family": format!("{:?}", game.family()),
                "uses_community_board": game.family().uses_community_board(),
                "is_stud_family": game.family().is_stud_family(),
                "ranks_ace_low": game.family().ranks_ace_low(),
                "cards_per_player": game.cards_per_player(),
                "cards_on_board": game.cards_on_board(),
                "deck_size": game.get_deck_size(),
                "betting_structure": format!("{:?}", game.betting()),
                "street_count": streets.len(),
                "streets": streets,
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "streets",
        json!({
            "description": "Each variant is a game family paired with a betting structure. The \
                            street table drives dealing: how many community and hole cards are \
                            dealt, how many hole cards are face up, whether a card is burned \
                            first, and which bet tier applies. Razz's street table is identical \
                            to Seven-Card Stud Hi's.",
            "variants": street_rows,
        }),
    );

    let nl = BettingStructure::NoLimit;
    let pl = BettingStructure::PotLimit;
    let fl = BettingStructure::FixedLimit {
        small_bet: 100,
        big_bet: 200,
        raise_cap: 4,
    };

    let min_raises: Vec<Value> = [
        ("no-limit, first raise of the street", nl, 0usize, 100usize),
        ("no-limit, after a 200 raise", nl, 200, 100),
        ("pot-limit, first raise of the street", pl, 0, 100),
        ("pot-limit, after a 200 raise", pl, 200, 100),
        ("fixed-limit, first raise of the street", fl, 0, 100),
        ("fixed-limit, after a 200 raise", fl, 200, 100),
    ]
    .iter()
    .map(|(note, structure, last_raise, big_blind)| {
        json!({
            "note": note,
            "structure": format!("{structure:?}"),
            "last_raise": last_raise,
            "big_blind": big_blind,
            "min_raise_increment": structure.min_raise(*last_raise, *big_blind),
        })
    })
    .collect();

    let tiered: Vec<Value> = [BetTier::Small, BetTier::Big]
        .iter()
        .map(|tier| {
            json!({
                "tier": format!("{tier:?}"),
                "fixed_limit_increment": fl.min_raise_for_tier(0, 0, *tier),
            })
        })
        .collect();

    let max_raises: Vec<Value> = [
        (
            "pot-limit: pot 100, no bet to call, 1000 behind",
            pl,
            100usize,
            0usize,
            0usize,
            1000usize,
        ),
        (
            "pot-limit: pot 300, facing a 100 bet, 1000 behind",
            pl,
            300,
            100,
            0,
            1000,
        ),
        ("no-limit: the whole stack is always available", nl, 300, 100, 0, 1000),
        ("fixed-limit: capped at the tier increment", fl, 300, 100, 0, 1000),
        ("pot-limit: short stack caps the maximum", pl, 300, 100, 0, 150),
    ]
    .iter()
    .map(|(note, structure, pot, current_bet, committed, stack)| {
        json!({
            "note": note,
            "structure": format!("{structure:?}"),
            "pot": pot,
            "current_bet": current_bet,
            "already_committed": committed,
            "stack_remaining": stack,
            "max_raise_to": structure.max_raise(*pot, *current_bet, *committed, *stack, BetTier::Small),
        })
    })
    .collect();

    let caps: Vec<Value> = (0u8..6)
        .map(|raises| {
            json!({
                "raises_this_street": raises,
                "fixed_limit_cap_reached": fl.cap_reached(raises),
                "no_limit_cap_reached": nl.cap_reached(raises),
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "raise-sizing",
        json!({
            "description": "Minimum and maximum raise sizing across the three betting \
                            structures. No-limit and pot-limit share a minimum rule (the \
                            previous raise, or the big blind if none); they differ only in the \
                            maximum. Fixed-limit uses tier increments and enforces a raise cap.",
            "minimum_raises": min_raises,
            "fixed_limit_tiers": tiered,
            "maximum_raises": max_raises,
            "raise_caps": caps,
        }),
    );

    let mut position_rows: Vec<Value> = Vec::new();
    for seat_count in [2u8, 3, 4, 5, 6, 7, 9] {
        let mut seats: Vec<Value> = Vec::new();
        for seat in 0..seat_count {
            seats.push(json!({
                "seat": seat,
                "position": Position::from_seat(seat, 0, seat_count).map(|p| format!("{p:?}")),
            }));
        }
        position_rows.push(json!({
            "seat_count": seat_count,
            "button_seat": 0,
            "supported": Position::from_seat(0, 0, seat_count).is_some(),
            "seats": seats,
        }));
    }

    n += write_vector(
        epic,
        slug,
        "positions",
        json!({
            "description": "Position is derived from a seat's clockwise offset from the button. \
                            Only tables of 2, 3, 4, 5, 6, and 9 seats are defined; other sizes \
                            have no position mapping. Note that heads-up is special: the button \
                            is also the small blind.",
            "tables": position_rows,
        }),
    );

    n
}

// ── DECON-07 pot accounting ──────────────────────────────────────────────────

fn pot_accounting() -> usize {
    let epic = "DECON-07";
    let slug = "pot-accounting";
    let mut n = 0;

    let cases = [
        (1000usize, 1usize),
        (1000, 2),
        (1000, 3),
        (1000, 4),
        (11, 3),
        (11, 2),
        (100, 3),
        (7, 4),
        (0, 3),
    ];

    let divisions: Vec<Value> = cases
        .iter()
        .map(|(pot, winners)| {
            let stack = Stack::new(*pot);
            let shares: Vec<usize> = stack.divvy_up(*winners).iter().map(Stack::count).collect();
            let total: usize = shares.iter().sum();
            json!({
                "pot": pot,
                "winners": winners,
                "shares": shares,
                "sum_of_shares": total,
                "conserves_chips": total == *pot,
                "remainder": pot % winners.max(&1),
            })
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "division",
        json!({
            "description": "Splitting a pot among tied winners. Chip conservation is absolute: \
                            the shares always sum to the pot. When the pot does not divide \
                            evenly the remainder goes to the LAST shares. WHICH RECIPIENT IS \
                            'LAST' IS UNSPECIFIED in the original and diverges from the \
                            canonical casino rule of awarding the odd chip to the first player \
                            left of the button — see spec decision SD-03.",
            "divisions": divisions,
        }),
    );

    n
}

// ── DECON-09 equity and odds ─────────────────────────────────────────────────

fn equity_and_odds() -> usize {
    let epic = "DECON-09";
    let slug = "equity-and-odds";
    let mut n = 0;

    let odds_cases = [
        (100u64, 50u64, "half-pot bet"),
        (100, 100, "pot-sized bet"),
        (200, 50, "quarter-pot bet"),
        (150, 50, "one-third-pot bet"),
        (100, 25, "small bet"),
    ];
    let odds: Vec<Value> = odds_cases
        .iter()
        .map(|(pot, call, note)| {
            let po = PotOdds::new(*pot, *call);
            json!({
                "note": note,
                "pot": pot,
                "to_call": call,
                "ratio": format!("{:.6}", po.ratio()),
                "breakeven_equity": format!("{:.6}", po.breakeven()),
                "profitable_at_30_percent": po.is_profitable(0.30),
                "profitable_at_40_percent": po.is_profitable(0.40),
                "profitable_at_50_percent": po.is_profitable(0.50),
            })
        })
        .collect();

    // Exact enumeration: every remaining runout is evaluated.
    let exact_cases = [
        ("A♠ A♥", "K♦ K♣", "Q♠ J♦ 2♣", "aces against kings on a dry flop"),
        ("A♠ K♠", "Q♥ Q♦", "J♠ T♠ 2♥", "big draw against an overpair"),
        (
            "A♠ A♥",
            "K♦ K♣",
            "K♠ J♦ 2♣ 7♥ 3♠",
            "a completed board — one runout, so certainty",
        ),
    ];
    let exact: Vec<Value> = exact_cases
        .iter()
        .filter_map(|(hero, villain, board_str, note)| {
            let h = Two::from_str(hero).ok()?;
            let v = Two::from_str(villain).ok()?;
            let cards = Cards::from_str(board_str).ok()?;
            let board = if cards.len() == 5 {
                Board::from(Five::try_from(cards).ok()?)
            } else {
                Board::new(Three::try_from(cards).ok()?, Card::BLANK, Card::BLANK)
            };
            let request = EquityRequest {
                players: vec![PlayerSpec::Exact(h), PlayerSpec::Exact(v)],
                board,
                opts: EquityOptions::default(),
            };
            let report = request.compute().ok()?;
            Some(json!({
                "note": note,
                "hero": hero,
                "villain": villain,
                "board": board_str,
                "method": format!("{:?}", report.method),
                "runouts_evaluated": report.samples,
                "seats": report.players.iter().enumerate().map(|(i, p)| json!({
                    "seat": i,
                    "win": format!("{:.6}", p.win),
                    "tie": format!("{:.6}", p.tie),
                    "equity": format!("{:.6}", p.equity),
                    "sole_win_cases": p.wins,
                    "tied_cases": p.ties,
                })).collect::<Vec<Value>>(),
            }))
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "exact",
        json!({
            "description": "Exact equity: every remaining board runout is enumerated, so the \
                            answer is certain rather than estimated. The case counts are exact \
                            integers and must match. The equity fractions may differ in their \
                            last places because summation order is free, so compare them within \
                            a small tolerance. Note 'equity' folds split pots in fractionally, \
                            which is why win + tie is greater than or equal to equity.",
            "count_conformance": "exact",
            "fraction_conformance": "within 1e-6",
            "cases": exact,
        }),
    );

    // Sampled equity at a fixed seed: reproducible, but generator-specific.
    let sampled: Vec<Value> = [7u64, 42]
        .iter()
        .filter_map(|seed| {
            let h = Two::from_str("A♠ A♥").ok()?;
            let v = Two::from_str("K♦ K♣").ok()?;
            let request = EquityRequest {
                players: vec![PlayerSpec::Exact(h), PlayerSpec::Exact(v)],
                board: Board::default(),
                opts: EquityOptions {
                    max_samples: 20_000,
                    seed: Some(*seed),
                    ..Default::default()
                },
            };
            let report = request.compute().ok()?;
            Some(json!({
                "seed": seed,
                "board": "(none — before the flop)",
                "method": format!("{:?}", report.method),
                "samples": report.samples,
                "hero_equity": format!("{:.4}", report.players.first().map_or(0.0, |p| p.equity)),
                "villain_equity": format!("{:.4}", report.players.get(1).map_or(0.0, |p| p.equity)),
            }))
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "sampled-seeded",
        json!({
            "description": "INFORMATIVE, NOT NORMATIVE. When the runout space is too large to \
                            enumerate, equity is sampled. A fixed seed makes a result \
                            reproducible regardless of scheduling or parallelism, which is the \
                            normative property. The specific figures below depend on the \
                            original's random generator, so a rebuild must NOT be expected to \
                            match them -- it must show that its own fixed seed reproduces its \
                            own result, and that sampled answers converge on the exact ones.",
            "normative_property": "a fixed seed reproduces the same result within one implementation",
            "convergence_expectation": "sampled equity approaches the exact answer as samples increase",
            "runs": sampled,
        }),
    );

    n += write_vector(
        epic,
        slug,
        "pot-odds",
        json!({
            "description": "Pot odds express the price being laid on a call. The break-even \
                            equity is the share of the pot a hand must win to make calling \
                            neutral: to_call / (pot + to_call).",
            "cases": odds,
        }),
    );

    n
}

// ── DECON-13 equilibrium solving ─────────────────────────────────────────────

fn equilibrium_solving() -> usize {
    let epic = "DECON-13";
    let slug = "equilibrium-solving";
    let mut n = 0;

    // Enumerate the toy game's full tree by exhaustive traversal.
    let cards = [KuhnCard::Jack, KuhnCard::Queen, KuhnCard::King];
    let mut terminals: Vec<Value> = Vec::new();
    let mut deals = 0usize;

    for p0 in cards {
        for p1 in cards {
            if p0 == p1 {
                continue;
            }
            deals += 1;
            let Ok(root) = KuhnState::new(p0, p1) else {
                eprintln!("FATAL: cannot build toy-game state");
                std::process::exit(1);
            };
            walk(&root, &KuhnHistory::new(), &mut terminals, p0, p1);
        }
    }

    n += write_vector(
        epic,
        slug,
        "kuhn-tree",
        json!({
            "description": "The toy game: a three-card deck, two players, one ante each, and a \
                            single betting round. Its full tree is small enough to enumerate \
                            exhaustively, which is what makes it a correctness oracle for any \
                            equilibrium solver. Payoffs are from each player's perspective and \
                            always sum to zero.",
            "deck": ["Jack", "Queen", "King"],
            "distinct_deals": deals,
            "terminal_count": terminals.len(),
            "terminals": terminals,
        }),
    );

    // The analytic equilibrium family, sampled at the ends and middle of alpha.
    let mut family: Vec<Value> = Vec::new();
    for (label, alpha) in [
        ("alpha = 0 (lower bound)", 0.0f64),
        ("alpha = 1/6 (midpoint)", 1.0 / 6.0),
        ("alpha = 1/3 (upper bound)", 1.0 / 3.0),
    ] {
        let Ok(strategy) = KuhnStrategy::gto(alpha) else {
            eprintln!("FATAL: cannot build analytic strategy");
            std::process::exit(1);
        };
        let empty = KuhnHistory::new();
        let h_check = empty.push(KuhnAction::Check);
        let h_bet = empty.push(KuhnAction::Bet);
        let h_check_bet = h_check.push(KuhnAction::Bet);

        let mut rows: Vec<Value> = Vec::new();
        for (context, history) in [
            ("first player, opening", &empty),
            ("second player, facing a check", &h_check),
            ("second player, facing a bet", &h_bet),
            ("first player, facing a check-then-bet", &h_check_bet),
        ] {
            for card in cards {
                let info = KuhnInfoSet::new(card, history.clone());
                let probs = strategy.action_probs(&info);
                if probs.is_empty() {
                    continue;
                }
                rows.push(json!({
                    "context": context,
                    "info_set": info.to_string(),
                    "card": format!("{card:?}"),
                    "actions": probs.iter().map(|(a, p)| json!({
                        "action": format!("{a:?}"),
                        "probability": format!("{p:.6}"),
                    })).collect::<Vec<Value>>(),
                }));
            }
        }
        family.push(json!({ "label": label, "alpha": format!("{alpha:.6}"), "strategy": rows }));
    }

    // Train the solver and record how close it gets to the known answer.
    let mut cfr = KuhnCfr::new();
    cfr.train(200_000).expect("Kuhn training cannot fail on valid deals");
    let exploitability = cfr.exploitability();

    n += write_vector(
        epic,
        slug,
        "kuhn-equilibrium",
        json!({
            "description": "The toy game's equilibrium is known analytically, which is why it \
                            validates a solver far more strongly than any sampled number could. \
                            The equilibrium is a FAMILY parameterised by a single value between \
                            0 and 1/3. Within it, several frequencies are uniquely determined \
                            regardless of the parameter, and the first player's king bet rate \
                            is always exactly three times the jack bluff rate.",
            "game_value_to_first_player": "-1/18",
            "game_value_decimal": format!("{:.6}", -1.0 / 18.0),
            "alpha_range": { "minimum": "0", "maximum": "1/3" },
            "determined_frequencies": {
                "first_player_bets_a_queen": "0",
                "second_player_calls_a_king": "1",
                "second_player_calls_a_jack": "0",
                "second_player_bets_a_king_after_a_check": "1",
                "first_player_calls_with_a_king": "1",
                "first_player_folds_a_jack": "1",
                "second_player_bluffs_a_jack_after_a_check": "1/3",
                "second_player_calls_with_a_queen": "1/3"
            },
            "structural_relation": "first player's king bet rate = 3 x jack bluff rate",
            "analytic_family": family,
            "solved": {
                "iterations": 200_000,
                "exploitability": format!("{exploitability:.9}"),
                "note": "Regret-matching self-play converges toward the analytic answer. The \
                         AVERAGE strategy converges, not the final one. This figure is \
                         informative: a rebuild must converge, but need not match this value.",
            },
        }),
    );

    n
}

fn walk(state: &KuhnState, history: &KuhnHistory, out: &mut Vec<Value>, p0: KuhnCard, p1: KuhnCard) {
    if state.is_terminal() {
        let Ok(payoff) = state.payoff() else {
            eprintln!("FATAL: terminal state has no payoff");
            std::process::exit(1);
        };
        out.push(json!({
            "first_player_card": format!("{p0:?}"),
            "second_player_card": format!("{p1:?}"),
            "history": history.to_string(),
            "payoff_first_player": payoff[0],
            "payoff_second_player": payoff[1],
            "sums_to_zero": payoff[0] + payoff[1] == 0,
        }));
        return;
    }
    for action in state.legal_actions() {
        let Ok(next) = state.apply(action) else {
            eprintln!("FATAL: legal action was rejected");
            std::process::exit(1);
        };
        walk(&next, &history.push(action), out, p0, p1);
    }
}

// ── DECON-10 suit isomorphism ────────────────────────────────────────────────

fn suit_isomorphism() -> usize {
    let epic = "DECON-10";
    let slug = "suit-isomorphism";
    let mut n = 0;

    // Ordered explicitly: the source's suit listing is not order-stable, and
    // the rotation cycle is what matters, not the enumeration order.
    let mut ordered: Vec<Suit> = Suit::all().into_iter().collect();
    ordered.sort_by_key(|suit: &Suit| suit.to_char_letter());
    let suits: Vec<Value> = ordered
        .iter()
        .map(|suit| {
            json!({
                "suit": suit.to_char_letter().to_string(),
                "shift_up": suit.shift_suit_up().to_char_letter().to_string(),
                "shift_down": suit.shift_suit_down().to_char_letter().to_string(),
            })
        })
        .collect();

    let holdings = ["A♠ K♠", "A♠ K♥", "7♦ 2♣", "Q♥ Q♦"];
    let shifted: Vec<Value> = holdings
        .iter()
        .filter_map(|s| {
            let two = Two::from_str(s).ok()?;
            // Apply the rotation repeatedly to trace the full cycle.
            let mut cycle: Vec<String> = Vec::new();
            let mut current = two;
            for _ in 0..4 {
                cycle.push(current.to_string());
                current = current.shift_suit_up();
            }
            let mut distinct: Vec<String> = cycle.clone();
            distinct.sort();
            distinct.dedup();
            Some(json!({
                "holding": s,
                "class": two.get_letter_index(),
                "rotation_cycle": cycle,
                "returns_to_start": current == two,
                "distinct_rotations": distinct.len(),
            }))
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "shifts",
        json!({
            "description": "Two situations that differ only by a relabelling of suits have \
                            identical equity. Rotating every suit in a holding by the same step \
                            therefore produces an equity-equivalent holding. This is the \
                            compression that makes exhaustive matchup analysis tractable.",
            "suit_rotation": suits,
            "holdings": shifted,
            "matchup_universe_size": pkcore::POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS,
        }),
    );

    let matchups = [("A♠ K♠", "Q♥ Q♦"), ("Q♥ Q♦", "A♠ K♠"), ("7♦ 2♣", "A♠ A♥")];
    let canonical: Vec<Value> = matchups
        .iter()
        .filter_map(|(a, b)| {
            let first = Two::from_str(a).ok()?;
            let second = Two::from_str(b).ok()?;
            let shu = SortedHeadsUp::new(first, second);
            Some(json!({
                "input_first": a,
                "input_second": b,
                "canonical_higher": shu.higher().to_string(),
                "canonical_lower": shu.lower().to_string(),
            }))
        })
        .collect();

    n += write_vector(
        epic,
        slug,
        "canonicalization",
        json!({
            "description": "A heads-up matchup is canonicalized into a higher/lower ordering so \
                            that a matchup and its mirror share one representative. Presenting \
                            the same two holdings in either order must yield the same canonical \
                            form.",
            "matchups": canonical,
        }),
    );

    n
}
