use std::str::FromStr;

use cucumber::gherkin::Step;
use cucumber::{given, then, when};
use pkcore::arrays::sliced::BoxedCards;
use pkcore::prelude::*;

use crate::world::{PokerWorld, squash};
use pkcore::casino::winnings::Winnings;

// ── setting the table ─────────────────────────────────────────────────────

/// Seats players from a data table with `name` and `chips` columns and an
/// optional `cards` column. Seats are filled in row order, from seat 0; the
/// first row is the button.
#[given(expr = "a no-limit hold'em table with blinds of {int}\\/{int} and these players:")]
fn nlh_table(world: &mut PokerWorld, small: usize, big: usize, step: &Step) {
    let rows = &step.table.as_ref().expect("the players are given as a data table").rows;
    let header = &rows[0];
    let column = |name: &str| header.iter().position(|h| h == name);
    let name_col = column("name").expect("a `name` column");
    let chips_col = column("chips").expect("a `chips` column");
    let cards_col = column("cards");

    let seats: Vec<Seat> = rows[1..]
        .iter()
        .map(|row| {
            let chips: usize = row[chips_col]
                .replace(',', "")
                .parse()
                .expect("chips are a whole number");
            let mut seat = Seat::new(Player::new_with_chips(row[name_col].clone(), chips));
            if let Some(col) = cards_col.filter(|&c| !row[c].trim().is_empty()) {
                seat.cards = BoxedCards::from_str(&row[col])
                    .unwrap_or_else(|e| panic!("{:?} are not hole cards: {e:?}", row[col]));
            }
            seat
        })
        .collect();

    let table = Table::nlh_from_seats(Seats::new(seats), ForcedBets::new(small, big));
    world.chips_at_start = table.table_chip_count();
    world.table = Some(table);
}

/// Puts `top` on top of the deck in the order the dealer draws it — burn,
/// three flop cards, burn, turn, burn, river — with the rest of the unused
/// cards beneath, so the board runs out exactly as the scenario says.
#[given(expr = "the deck is stacked to deal {string}")]
fn stack_the_deck(world: &mut PokerWorld, top: String) {
    let table = world.table_mut();
    let mut used = Cards::from_str(&top).expect("the stacked cards parse");
    for i in 0..table.seats.size() {
        if let Some(seat) = table.seats.get_seat(i) {
            used.insert_all(&seat.cards.cards());
        }
    }
    let mut deck = Cards::from_str(&top).expect("the stacked cards parse");
    deck.insert_all(&Cards::deck_minus(&used));
    table.deck = deck;
}

// ── dealing ───────────────────────────────────────────────────────────────

#[when("the blinds are posted")]
fn post_blinds(world: &mut PokerWorld) {
    world.table_mut().act_forced_bets().expect("the blinds post");
}

#[when("the hole cards are dealt")]
fn deal_hole_cards(world: &mut PokerWorld) {
    world.table_mut().deal_cards_to_seats().expect("hole cards deal");
}

#[when("the bets are brought in")]
fn bring_it_in(world: &mut PokerWorld) {
    world.table_mut().bring_it_in().expect("bets are brought in");
}

#[when("the board is run out")]
fn run_out(world: &mut PokerWorld) {
    let table = world.table_mut();
    table.bring_it_in().expect("bets are brought in");
    table.deal_flop().expect("the flop deals");
    table.deal_turn().expect("the turn deals");
    table.deal_river().expect("the river deals");
}

#[when("the hand ends")]
fn end_hand(world: &mut PokerWorld) {
    world.winnings = Some(world.table_mut().end_hand());
}

// ── player actions ────────────────────────────────────────────────────────

/// Every action a player can take, as the feature files spell them.
fn act(world: &mut PokerWorld, name: &str, action: &str) -> Result<usize, PKError> {
    let seat = world.seat_of(name);
    let table = world.table_mut();
    let words: Vec<&str> = action.split_whitespace().collect();
    let amount = || -> usize {
        words
            .last()
            .and_then(|w| w.replace(',', "").parse().ok())
            .expect("an amount")
    };
    match words.as_slice() {
        ["folds"] => table.act_fold(seat),
        ["checks"] => table.act_check(seat),
        ["calls"] => table.act_call(seat),
        ["goes", "all", "in"] => table.act_all_in(seat),
        ["bets", _] => table.act_bet(seat, amount()),
        ["raises", "to", _] => table.act_raise(seat, amount()),
        _ => panic!("unknown action {action:?}"),
    }
}

#[when(regex = r"^(\w+) (folds|checks|calls|goes all in|bets [\d,]+|raises to [\d,]+)$")]
fn player_acts(world: &mut PokerWorld, name: String, action: String) {
    let result = act(world, &name, &action);
    if let Err(e) = &result {
        panic!("{name} {action}: refused with {e:?}");
    }
    world.last_action = Some(result);
}

#[when(regex = r"^(\w+) tries to (fold|check|call|go all in|bet [\d,]+|raise to [\d,]+)$")]
fn player_tries(world: &mut PokerWorld, name: String, action: String) {
    // "tries to fold" → "folds", "tries to go all in" → "goes all in", …
    let action = match action.split_once(' ') {
        Some(("go", rest)) => format!("goes {rest}"),
        Some((verb, rest)) => format!("{verb}s {rest}"),
        None => format!("{action}s"),
    };
    world.last_action = Some(act(world, &name, &action));
}

#[when("everyone still to act goes all in")]
fn everyone_all_in(world: &mut PokerWorld) {
    let table = world.table_mut();
    // Bounded by the seat count: each shove removes one seat from the action.
    for _ in 0..table.seats.size() {
        if table.seats.is_betting_complete() {
            break;
        }
        let seat = table.next_to_act();
        table.act_all_in(seat).expect("the shove is accepted");
    }
    assert!(
        table.seats.is_betting_complete(),
        "betting is still open after every seat shoved"
    );
}

// ── positions and amounts ─────────────────────────────────────────────────

fn position(world: &PokerWorld, role: &str) -> u8 {
    let table = world.table();
    match squash(role).as_str() {
        "smallblind" => table.determine_small_blind(),
        "bigblind" => table.determine_big_blind(),
        "undertheguns" | "utg" | "underthegun" => table.determine_utg(),
        _ => panic!("unknown position {role:?}"),
    }
}

#[then(expr = "{word} is the {string}")]
fn is_position(world: &mut PokerWorld, name: String, role: String) {
    assert_eq!(world.seat_of(&name), position(world, &role), "{name} is not the {role}");
}

#[then(expr = "the action is on {word}")]
fn action_on(world: &mut PokerWorld, name: String) {
    let next = world.table().next_to_act();
    let handle = &world.table().seats.get_seat(next).expect("a seat").player.handle;
    assert_eq!(&name, handle, "the action is on {handle}");
}

#[then(expr = "{word} has {int} chips in front")]
fn chips_in_front(world: &mut PokerWorld, name: String, chips: usize) {
    let seat = world.seat_of(&name);
    assert_eq!(chips, world.table().seats.get_seat(seat).expect("a seat").player.bet);
}

#[then(expr = "{word} needs {int} to call")]
fn needs_to_call(world: &mut PokerWorld, name: String, chips: usize) {
    let seat = world.seat_of(&name);
    assert_eq!(chips, world.table().to_call(seat));
}

#[then(expr = "the minimum raise is to {int}")]
fn min_raise_to(world: &mut PokerWorld, chips: usize) {
    assert_eq!(chips, world.table().min_raise_to());
}

#[then(expr = "{word} has {int} chips")]
fn has_chips(world: &mut PokerWorld, name: String, chips: usize) {
    let seat = world.seat_of(&name);
    assert_eq!(chips, world.table().seats.get_seat(seat).expect("a seat").player.chips);
}

#[then("every chip is still on the table")]
fn conserved(world: &mut PokerWorld) {
    assert_eq!(world.chips_at_start, world.table().table_chip_count());
}

// ── outcomes ──────────────────────────────────────────────────────────────

#[then("the action is refused as out of turn")]
fn refused_out_of_turn(world: &mut PokerWorld) {
    let result = world.last_action.as_ref().expect("nobody tried to act");
    assert!(
        matches!(result, Err(PKError::TableActionOutOfOrder(_))),
        "got {result:?}"
    );
}

#[then("the action is refused")]
fn refused(world: &mut PokerWorld) {
    let result = world.last_action.as_ref().expect("nobody tried to act");
    assert!(result.is_err(), "the action was accepted: {result:?}");
}

#[then("the hand is over")]
fn hand_over(world: &mut PokerWorld) {
    assert!(world.table().is_game_over());
}

#[then("the hand is not over")]
fn hand_not_over(world: &mut PokerWorld) {
    assert!(!world.table().is_game_over());
}

fn winnings(world: &PokerWorld) -> &Winnings {
    match world.winnings.as_ref().expect("the hand has not ended") {
        Ok(winnings) => winnings,
        Err(e) => panic!("the hand failed to end: {e:?}"),
    }
}

#[then(expr = "the winnings list {int} award(s)")]
fn award_count(world: &mut PokerWorld, awards: usize) {
    let winnings = winnings(world);
    assert_eq!(awards, winnings.len(), "{winnings}");
}

/// An award is one `PotWin`: chips, the seats sharing them, and the hand
/// credited with them.
#[then(expr = "{word} is awarded {int} chips with a(n) {string}")]
fn awarded(world: &mut PokerWorld, name: String, chips: usize, hand: String) {
    let seat = world.seat_of(&name);
    let winnings = winnings(world);
    let found = winnings.vec().iter().any(|win| {
        win.equity.chips == chips
            && win.equity.seats.0 & (1 << seat) != 0
            && squash(&format!("{:?}", win.eval.hand_rank.name)) == squash(&hand)
    });
    assert!(found, "no award of {chips} to {name} with a {hand}: {winnings}");
}

#[then(expr = "the largest award is credited to a(n) {string}")]
fn largest_award(world: &mut PokerWorld, hand: String) {
    // `Winnings` keeps its awards sorted largest first.
    let actual = format!("{:?}", winnings(world).first().eval.hand_rank.name);
    assert_eq!(
        squash(&hand),
        squash(&actual),
        "the largest award is credited to a {actual}"
    );
}
