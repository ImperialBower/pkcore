# EPIC 7 - Transposition

Right now, we've been able to do some pretty complex analysis of hand comparisons. The thing is, why repeat the calculation
if you've done it already? We need a way to store our results. For that, we're going to need a database. 

Each calculation is going to take time, but, in poker, unlike games like Bridge and Spades, the suits of the cards
are equal, so if we do the calculation once, the results will be the same for each other suit. For example:

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

    A♠ A♥ vs K♦ K♣
    A♠ A♦ vs K♥ K♣
    A♠ A♣ vs K♥ K♦
    A♥ A♦ vs K♠ K♣
    A♥ A♣ vs K♠ K♦
    A♦ A♣ vs K♠ K♥

Each of these matchups have the same odds of winning preflop. Is there a way I can do the calculations once, and then 
apply them to every possible variation?

At first I was hoping that simply shifting the suits in one direction four times would do it. `A♠ A♥ vs K♦ K♣` would shift
to `A♠ A♣ vs K♥ K♦`, etc. This assumes a relationship between the suits that is more in tune with card games like Bridge,
where suits can outrank each other. Spades beats hearts beats diamonds beats Clubs. While this doesn't apply to Poker;
a Royal Flush with Clubs (`A♣ K♣ Q♣ J♣ T♣`) is just a good as a Royal Flush with Spades (`A♠ K♠ Q♠ J♠ T♠`), by transposing
the hands of each player three times you can ensure that you cover other hands that would generate the exact same results.

The problem is that this would only cover four of the six variations: 

    A♠ A♥ vs K♦ K♣
    A♠ A♣ vs K♥ K♦
    A♦ A♣ vs K♠ K♥
    A♥ A♦ vs K♠ K♣

This is because the suits in each of the hands are only one step removed. Spades to hearts and clubs. Hearts to diamonds
and spades, etc. We're missing the ones that are one removed: `A♠ A♦ vs K♥ K♣` and `A♥ A♣ vs K♠ K♦`.