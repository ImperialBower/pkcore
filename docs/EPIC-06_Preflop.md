# Preflop

Now that we've gotten some basic concurrency to spead up our odds calculations at the flop, 
we're ready to start on the hardest calculation: Odds preflop. 

Preflop is where the petal hits the metal.

When we were determining the odds at the flop heads up, we were iterating over 903 different unique
combination of cards that could be dealt. At the deal, that number increases to 1,712,304.
The effort to determine the exact odds is increasing geometrically. 

Since these calculations are so heavy, we are going to need a plan. In fact, I'm thinking we're
going to need several plans:

1. Store the absolute results in some sort of DB, either flat file or simple text thing.
2. Some method of distilling down odds based on basic patters, such as two over cards vs pair, etc.

For this, I'm feeling the need to have a very simple way to store combinations of cards.

## Self doubt

I've walked down these corridors before. I feel like I am repeating myself. Still, I have a path
so I need to stick to it. I want a way to store Cards into a single number for easy lookups. It's
a radical form of serialization. `Cards` go in, `Bard` goes out. `Bard` goes in, `Cards` come out.

## Card to Bard

## Cards to Bard

### Bard to Cards

OK, now that we can collapse a collection of Cards into a single `Bard` integer, let's do the reverse,
deconstructing a binary `Bard` into a `Cards` collection. Note: one bit of information that will be
lost when going back to `Cards` from a Bard is the order of the `Cards`.