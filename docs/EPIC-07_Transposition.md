# EPIC 7 - Transposition

Right now, we've been able to do some pretty complex analysis of hand comparisons. The thing is, why repeat the calculation
if you've done it already? We need a way to store our results. For that, we're going to need a database. 

Each calculation is going to take time, but, in poker, unlike games like Bridge and Spades, the suits of the cards
are equal, so if we do the calculation once, the results will be the same for each other suit. When Cactus Kev wrote 
about [his evaluator](https://suffe.cool/poker/evaluator.html) he called the difference unique vs. distinct. For instance,
while there are 40 unique straight flushes, there are only 10 distinct ones, since a royal flush of all Spades ties
with a royal flush of Hearts.

It gets a little, no, let me lie, it gets a LOT more complicated when we are doing analysis comparing one hand
against another. For example:

    With the hand A♠ A♥ vs K♦ K♣, the aces win 81.06% of the time, and the Kings win 18.55% of the time, with 0.38% draws.
    Now if I were to reverse the suits of the cards; make it A♦ A♣ vs K♠ K♥, the results would be the same. It doesn't matter
    what the suits of the cards are, just their value AND if any of the cards in all of the players hands are of the same 
    suit. 

## Covered

With the first example, neither player has cards of the same suit, but the odds change slightly when the do. Let's see
what happens when we change the K♦ to the K♠. Now, the aces win 81.71% of the time, and the kings win 17.82% of the time, 
with them drawing 0.46%. This is because we have removed the ability for the kings to win with a spades flush, because
the aces have the nut flush with the ace. 

_There is one spades flush that the kings would win. Can you guess what it is?_

Also, since no one has a diamond, if there is a diamond flush on the board, both players will draw. 

In heads up, when the players share one card of the same suit, I call that covered, in homage to the Waffle House vernacular
of ordering your hash browns covered in melted cheese.

## Smothered

When both players have cards of the same suits, I call that smothered, as in getting your hash browns with diced onions.
In this case, the aces odds go up to 82.36%, and the kings go down to 17.09%, with them drawing 0.54% of the time.

For completeness, I shall call when none of their cards match as scattered, after having your hash browns spread out on 
the griddle when they're cooked.  

## Beyond Pairs

Now this applies to any combination of hands. A♠ K♠ vs Q♠ J♠ has the same odds of winning as A♦ K♦ vs Q♦ J♦. 

## Transposition

Here's the thing... if the odds are the same no matter what the actual suits are, why do I have to do the complex
calculation of preflop odds for each of the possible suit variations? 

    A♠ A♥ vs K♠ K♣ // COVERED 
    A♠ A♥ vs K♠ K♥ // SMOTHERED
    A♠ A♥ vs K♦ K♣ 

    A♠ A♦ vs K♥ K♣
    A♠ A♣ vs K♥ K♦
    A♥ A♦ vs K♠ K♣
    A♥ A♣ vs K♠ K♦
    A♦ A♣ vs K♠ K♥

Each of these matchups have the same odds of winning preflop. Is there a way I can do the calculations once, and then 
apply them to every possible variation?

At first I was hoping that simply shifting the suits in one direction four times would do it. 

    The concept of shifting as it applies to Card Suits turns them into what is called a bounded value. The idea is
    that when you have a value that reaches its bondary, the next value will be what's at the other end. For
    an Unsigned Integer bounded at 5, when I add 1 to it, the value becomes 0. Subtract 1 from 0 and the result is 5.

    So for Suits, the boundaries are Spades (4) at the top, and Clubs (1) at the bottom, so if I add one to Spades, I
    would get Clubs, and if I subtracted one from Clubs, I would get Spades. 

    One of the first crates I ever produced was [Bint](https://crates.io/crates/bint), a very simple Bounded Integer. I
    was playing around with embedded Rust on an STM32 educational board with one of those circular LEDs on it, and I 
    wanted a simple way to make the lights go around in circles, so I created the Bint library to take care of it. A 
    much, much better example can be found in the [bounded-integer](https://crates.io/crates/bounded-integer) crate.

`A♠ A♥ vs K♦ K♣` would shift to `A♠ A♣ vs K♥ K♦`, etc. This assumes a relationship between the suits that is more in tune with card games like Bridge,
where suits can outrank each other. Spades beats hearts beats diamonds beats Clubs. While this doesn't apply to Poker;
a Royal Flush with Clubs (`A♣ K♣ Q♣ J♣ T♣`) is just a good as a Royal Flush with Spades (`A♠ K♠ Q♠ J♠ T♠`), by transposing
the hands of each player three times you can ensure that you cover other hands that would generate the exact same results.

The problem is that this would only cover four of the six variations. `A♠ A♥ vs K♦ K♣` shifts to `A♠ A♣ vs K♥ K♦` shifts
to `A♦ A♣ vs K♠ K♥` shifts to `A♥ A♦ vs K♠ K♣` shifts back to `A♠ A♥ vs K♦ K♣`. This is because the suits in each of the
hands are only one step removed. Spades to Hearts and Clubs. Hearts to Diamonds and Spades, etc. We're missing the ones 
that are one removed: `A♠ A♦ vs K♥ K♣` and `A♥ A♣ vs K♠ K♦`.

This is still pretty good. Question: could we do better? How can cover all of the possibilities? 

Now at this point you make be thinking, _dude, you are seriously overthinking this shit_, and you, my friend, would be
correct. But let's be real; what aspect of civilization would have happened if there wasn't some nerd sitting in a tree
wondering what would happen if threw stuff at passing animals instead of trying to catch them with our hands. 

So, what's the difference? `A♠ A♦ vs K♥ K♣` shifts to `A♥ A♣ vs K♠ K♦` shifts back to `A♠ A♦ vs K♥ K♣`. That means that
for Hands that are two Suits removed there's only one direct transposition. 

What happens with we do indirect transposition? First, what do I mean by that? In this case, I'm thinking transposing
just one Card's suit. Which Card? I bet that it doesn't really matter. Let us see...

Starting with `A♠ A♥ vs K♦ K♣`, let's shift the card on the left for each hand and see what happens. Now we've got 
`A♣ A♥ vs K♠ K♣`. That doesn't work. Now the hands are sharing the same Club Suit aka Covered. That means they aren't
valued the same.


Algorithm:

* Determine the Suit distance between the two Cards.
    * If the two Cards are adjacent, shift each suit once over three more times.
    * ???

### Spike: Storage using Sled

In the past I used CSV to store analysis results. This was easy to implement, but since the table
contained millions of records, it took a long time to load into memory. Let's see if the embedded 
DB Sled will work for us. 

#### History: CSV

Before we experiment with Sled for storing results, let's go back in the time machine and see
how things worked using CSV as our data format. 

* Reflect on CSV
  * Why I love it
    * History
  * Faker library

##### CSV with Cards

First, let's try writing and reading a CSV file that has regular card strings as examples. Something
like this:

```
seven, five, hand_rank
6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠, 6♠ 6♥ 6♦ 5♠ 5♥, 271
5♦ 5♣ 9♣ 6♦ 5♥ 5♠ 8♠, 5♠ 5♥ 5♦ 5♣ 9♣, 124
```
