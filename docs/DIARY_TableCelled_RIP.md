# Diary: TableCelled RIP🪦

[`TableCelled`](https://github.com/ImperialBower/pkcore/blob/6992afcbd884a41c40ce4bce0cd8fc1ebb84cf76/src/casino/table_celled.rs#L120)
is one of my favorite coding accomplishments on this project. The idea that I could use Rust's
[Box struct](https://doc.rust-lang.org/std/boxed/index.html) heap superpowers to throw cards and
bets on and off of the table without having to deal with Rust's pesky mutability issues seemed 
cool as heck. Who cares if I'm adding an additional layer of code onto every piece of data in 
order to gain this programatic foo? It was cool, and it showed how easily I could bend Rust to
my will. 

Here's the thing. There's this trait that programmers new to Rust always seemed to have that I call,
_whatever the programming language I most code in envy._ In my case it was _Java envy._ It's easy
to just slip into the habits of your old languages when you code in Rust, like a comphy old shoe. Impls of 
structs would quickly fill with `get_shoes() -> Shoe` and `set_shoes(Shoe: shoe)` because that was what
you did. See, it's dangerous to expose all that state like your drawers are just hanging out like that...
it could cause... things. 

And so I weaved, and tested, and plotted, and twisted the code until it was as smooth as silk. I had a real
sense of accomplishment. This would make game play so much easier, and during early development, it did. 

Here's the thing: it burries issues. A bad borrow causes a panic, aka the program crashes, instead of a nice
compile time error, and since you've made passing around state so easy, the likelihood of a panic increases
quite a bit. Since everything is coming through cells, it's much slower. 

My first big Claude Code exercise was for it to rewrite TableCelled as a good ol' fashioned Rust struct with state
that you have to manage like a toddler in a shopping mall. It quickly took over as the goto engine for table
mechanics, and because there was essentially a fork between the two versions of the struct, it increased the complexity
and maintainability of the code by a factor of two. 

In the end, Table left TableCelled in the dust, which is why we are pouring one out for our homey today.

---

## The autopsy

`TableCelled` was removed by [EPIC-83](epics/EPIC-83_Table_Decelled.md) on
2026-08-24. Some numbers for the headstone:

| | |
|---|---|
| Lines deleted | 6,654 |
| Files deleted | 8 (`table_celled.rs`, `table_celled/` × 6, `player.rs`) |
| Public methods that existed only on the celled side | 44 |
| Tests deleted | 123 (109 unit, 14 integration) |
| Tests written to cover what those actually asserted | 13 |

Three things turned up on the way out that are worth keeping:

1. **It had been dealing to the wrong seat all along.** `TableCelled` started
   dealing *at* the button; poker deals to the button's **left**. Nobody
   noticed because the stacked test fixtures had been written against the bug,
   so both engines agreed with their own tests and with nothing else. Running
   "The Hand" on each is what caught it — the celled engine gave Gus Hansen the
   pot, the plain engine gave it to Daniel Negreanu, who had been dealt
   Hansen's 5♦ 5♣.

2. **The fork cost more than the cells did.** The `RefCell` overhead was never
   the problem. The problem was 44 methods with no twin and every rule needing
   two homes. A twin implementation is only a safety net if something actually
   compares their outputs, and for four months nothing did.

3. **Generics were not the way out.** The EPIC opened by asking whether one
   generic body could serve both. It could not: the trait would have had to
   abstract over `&self` versus `&mut self`, which is the whole difference
   between the two designs. Hiding it would have defeated the point.

The teaching value survives in
[`ANALYSIS_TableCelled_vs_Table.md`](ANALYSIS_TableCelled_vs_Table.md), which
is kept for exactly that reason.

