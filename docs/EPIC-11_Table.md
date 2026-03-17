# Table Epic


## Final Boss: Split Pots

Fucking Gemini:

> Side pots are the "final boss" of poker logic.

I hate it when it's right, but it's right. 

I wish my natural instinct was to just test drive through a problem, but my brain isn't generally
wired that way. The technique works on occasions, but it's not generally how I am wired. 

The problem: This is a complex state machine, that I am trying to isolate as a matrix. The problem
is that my allergies are kicking my ass, so I am in a permanent brain fog right now. 

The state that we are trying to isolate on is what I am calling round equity. 

- How much investment does each player have in the round?
- How can we isolate on specific split pots so that we ensure that every player is properly invested in the action in the hand?

The standard AI solutions have the following structure `(Vec<Player>, chips)`. This feels clunky to me. 
My natural instinct is to use bit flags to track the players for a specific investment level. 

Enter the `Seatbit`, a simple integer that stores a 1 for each bit reoresenting an active `Seat`. 

At first, feeling like shit, I decided to let AI take the wheel. I was trying to solve a specific problem:

- Players are all all-in.
- One player has more chips that all the others active in the hand.
- Currently, the Table was counting all the richest players chips in the hand.
- It needed to return the chips above the next richest player in the pot. 
- Their state needed to switch from `AllIn` to `Bet`. 

Here's the code it produced:

```rust
/// Returns the `x`-th highest committed bet level as `(Seatbit, amount)`.
///
/// The `depth` argument is zero-indexed:
/// - `0` => highest bet
/// - `1` => second-highest bet
/// - `2` => third-highest bet
///
/// If `depth` is greater than the number of occupied seats, this returns
/// `(Seatbit::default(), 0)`.
///
/// If the selected level amount is `0`, this also returns
/// `(Seatbit::default(), 0)`.
///
/// Seats tied at the same amount are combined into a single `Seatbit`.
///
/// # Examples
/// If seat 1 and seat 2 both have `9_000`, depth `1` returns
/// `(Seatbit::SEAT_1 | Seatbit::SEAT_2, 9_000)`.
#[must_use]
pub fn x_highest_bet(&self, depth: u8) -> (Seatbit, usize) {
    let mut ranked: Vec<(u8, usize)> = self
        .0
        .iter()
        .enumerate()
        .filter_map(|(seat_number, seat_cell)| {
            let seat = seat_cell.borrow();
            if seat.is_empty() {
                None
            } else {
                match u8::try_from(seat_number) {
                    Ok(snum) => Some((snum, seat.player.bet.count())),
                    Err(_) => None,
                }
            }
        })
        .collect();

    ranked.sort_unstable_by(|(seat_a, bet_a), (seat_b, bet_b)| bet_b.cmp(bet_a).then_with(|| seat_a.cmp(seat_b)));

    let mut levels: Vec<(Seatbit, usize)> = Vec::new();
    for (seat_number, amount) in ranked {
        if let Some((seatbits, existing_amount)) = levels.last_mut()
            && *existing_amount == amount
        {
            *seatbits |= Seatbit::from(seat_number);
        } else {
            levels.push((Seatbit::from(seat_number), amount));
        }
    }

    let (seatbits, amount) = levels.get(depth as usize).copied().unwrap_or((Seatbit::default(), 0));

    if amount == 0 {
        (Seatbit::default(), 0)
    } else {
        (seatbits, amount)
    }
}
```



