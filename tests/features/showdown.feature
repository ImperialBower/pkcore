@table @showdown
Feature: Showdowns, side pots and uncalled bets
  When players are all in for different amounts, each can win only what they
  matched: the short stack plays for the main pot, deeper stacks contest a
  side pot, and chips nobody could call go back to their owner. Whatever
  happens, no chip is created or lost.

  The deck is stacked so each board is known in advance. Cards are listed in
  the order the dealer draws them: burn, flop, burn, turn, burn, river.

  Rule: Heads-up the button posts the small blind and acts first

    Scenario: Two players
      Given a no-limit hold'em table with blinds of 50/100 and these players:
        | name | chips |
        | Ann  | 2000  |
        | Ben  | 2000  |
      When the blinds are posted
      And the hole cards are dealt
      Then Ann is the "small blind"
      And Ben is the "big blind"
      And the action is on Ann
      And Ann needs 50 to call

  Rule: A short stack wins only what it matched

    Scenario: The short stack wins and the deep stack's excess comes back
      Given a no-limit hold'em table with blinds of 50/100 and these players:
        | name  | chips | cards |
        | Deep  | 1000  | 7♦ 2♣ |
        | Short | 200   | A♠ A♥ |
      And the deck is stacked to deal "6♣ K♠ K♣ 9♦ 6♦ 8♠ 6♥ 4♥"
      When the blinds are posted
      And everyone still to act goes all in
      And the board is run out
      Then the hand is over
      When the hand ends
      Then Short is awarded 400 chips with a "two pair"
      And Short has 400 chips
      And Deep has 800 chips
      And every chip is still on the table

    @finding
    Scenario: An uncalled refund is reported as an award to the losing hand
      Deep's 800 unmatched chips come back to Deep, which is right. But
      `Winnings` lists the refund as a `PotWin` credited to Deep's losing
      pair of kings, indistinguishable from a pot Deep won — and, being the
      largest award, it is what `Winnings::first()` returns.
      Given a no-limit hold'em table with blinds of 50/100 and these players:
        | name  | chips | cards |
        | Deep  | 1000  | 7♦ 2♣ |
        | Short | 200   | A♠ A♥ |
      And the deck is stacked to deal "6♣ K♠ K♣ 9♦ 6♦ 8♠ 6♥ 4♥"
      When the blinds are posted
      And everyone still to act goes all in
      And the board is run out
      And the hand ends
      Then the winnings list 2 awards
      And Deep is awarded 800 chips with a "pair"
      And the largest award is credited to a "pair"

    Scenario: A tie with a short all-in splits only the matched chips
      Both players play four aces with a king kicker off the board.
      Given a no-limit hold'em table with blinds of 50/100 and these players:
        | name  | chips | cards |
        | Deep  | 1000  | 7♦ 2♣ |
        | Short | 200   | 4♦ 5♦ |
      And the deck is stacked to deal "6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥"
      When the blinds are posted
      And everyone still to act goes all in
      And the board is run out
      And the hand ends
      Then Deep has 1000 chips
      And Short has 200 chips
      And every chip is still on the table

  Rule: Three different stacks make a main pot and a side pot

    Background:
      Given a no-limit hold'em table with blinds of 50/100 and these players:
        | name    | chips | cards |
        | Rich    | 10000 | Q♦ Q♣ |
        | Poor    | 5000  | A♠ A♥ |
        | Average | 9000  | 4♣ 4♦ |

    Scenario: The best hand is the shortest stack
      Poor's aces win the 15,000 main pot; Rich's queens beat Average's
      fours for the 8,000 side pot; and the 1,000 of Rich's shove that
      nobody could call comes back to him.
      Given the deck is stacked to deal "2♥ K♠ 7♦ 2♣ 3♥ 9♥ 3♠ 5♠"
      When the blinds are posted
      And everyone still to act goes all in
      And the board is run out
      And the hand ends
      Then Poor is awarded 15000 chips with a "pair"
      And Poor has 15000 chips
      And Rich has 9000 chips
      And Average has 0 chips
      And every chip is still on the table

    Scenario: The middle stack hits a set and scoops both pots
      Given the deck is stacked to deal "2♥ K♠ 7♦ 2♣ 3♥ 9♥ 3♠ 4♥"
      When the blinds are posted
      And everyone still to act goes all in
      And the board is run out
      And the hand ends
      Then Average is awarded 23000 chips with a "three of a kind"
      And Average has 23000 chips
      And Rich has 1000 chips
      And Poor has 0 chips
      And every chip is still on the table

    @finding
    Scenario: Multiway, a winner's pots and refund are merged into one award
      Rich's 8,000 side pot and 1,000 refund arrive as a single 9,000 award,
      so the winnings cannot say which chips were won and which returned.
      Heads-up (above) the same refund is listed as an award of its own: the
      two showdown paths report the same event differently.
      Given the deck is stacked to deal "2♥ K♠ 7♦ 2♣ 3♥ 9♥ 3♠ 5♠"
      When the blinds are posted
      And everyone still to act goes all in
      And the board is run out
      And the hand ends
      Then the winnings list 2 awards
      And Rich is awarded 9000 chips with a "pair"
