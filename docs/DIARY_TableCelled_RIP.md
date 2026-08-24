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