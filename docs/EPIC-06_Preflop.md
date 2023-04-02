# Preflop

Now that we've gotten some basic concurrency to spead up our odds calculations at the flop, 
we're ready to start on the hardest calculation: Odds preflop. 

When we were determining the odds at the flop heads up, we were iterating over 903 different unique
combination of cards that could be dealt. At the deal, that number increases to 1,712,304.
The effort to determine the exact odds is increasing geometrically. 

Since these calculations are so heavy, we are going to need a plan. In fact, I'm thinking we're
going to need several plans:

1. Store the absolute results in some sort of DB, either flat file or simple text thing.
2. Some method of distilling down odds based on basic patters, such as two over cards vs pair, etc.

For this, I'm feeling the need to have a very simple way to store combinations of cards

Preflop is where the petal hits the metal. 