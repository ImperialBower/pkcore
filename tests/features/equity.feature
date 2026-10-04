@equity
Feature: Equity — who wins how often
  Given hole cards (or ranges, or unknown hands) and the board so far, pkcore
  deals out every remaining board and counts who wins. When there are too many
  runouts to enumerate — pre-flop, heads-up, is 1.7 million — it samples them
  instead, and a fixed seed makes the sample repeatable.

  Rule: On the turn and river the answer is counted, not estimated

    Scenario: A flush draw against a set on the turn
      Ace-king of spades has nine spades to come, but two of them pair the
      board and fill up the set. Seven rivers win, thirty-seven lose.
      Given seat 1 holds "A♠ K♠"
      And seat 2 holds "Q♥ Q♣"
      And the board is "Q♠ 7♠ 2♦ 3♣"
      When the equity is calculated
      Then the answer is exact
      And 44 runouts were evaluated
      And seat 1 wins 7 of them outright
      And seat 2 wins 37 of them outright
      And seat 1 has 15.91% equity
      And the seats' equity adds up to 100%

    Scenario: Drawing dead
      Given seat 1 holds "2♣ 3♦"
      And seat 2 holds "A♠ A♥"
      And the board is "A♦ A♣ K♠ K♥"
      When the equity is calculated
      Then seat 1 wins 0 of them outright
      And seat 1 has 0.0% equity
      And seat 2 has 100.0% equity

    Scenario: Both players play the board
      Given seat 1 holds "7♦ 2♣"
      And seat 2 holds "4♦ 5♦"
      And the board is "A♥ A♦ A♣ A♠ K♥"
      When the equity is calculated
      Then the answer is exact
      And 1 runout were evaluated
      And seat 1 ties 1 of them
      And seat 2 ties 1 of them
      And seat 1 has 50.0% equity
      And seat 2 has 50.0% equity

  Rule: On the flop every turn-and-river pair is enumerated

    Scenario: A flush draw against a set on the flop
      Forty-five unseen cards make C(45,2) = 990 turn-and-river runouts.
      Given seat 1 holds "A♠ K♠"
      And seat 2 holds "Q♥ Q♣"
      And the board is "Q♠ 7♠ 2♦"
      When the equity is calculated
      Then the answer is exact
      And 990 runouts were evaluated
      And seat 1 has between 20.0% and 35.0% equity
      And the seats' equity adds up to 100%

    Scenario: A range is sampled, even on the flop
      Six combinations of kings times 990 runouts would be cheap to count,
      but the engine enumerates only when every seat's cards are known.
      Given seat 1 holds "A♠ A♥"
      And seat 2 holds the range "KK"
      And the board is "2♣ 7♦ 9♥"
      And sampling is seeded with 3 and capped at 20000 samples
      When the equity is calculated
      Then the answer is a Monte Carlo estimate
      And seat 1 has between 85.0% and 95.0% equity
      And the seats' equity adds up to 100%

  Rule: Pre-flop the answer is sampled, and a seed makes it repeatable

    Scenario: Aces against kings
      Given seat 1 holds "A♠ A♥"
      And seat 2 holds "K♠ K♥"
      And sampling is seeded with 42 and capped at 20000 samples
      When the equity is calculated
      Then the answer is a Monte Carlo estimate
      And 20000 runouts were evaluated
      And seat 1 has between 79.0% and 85.0% equity
      And the seats' equity adds up to 100%

    Scenario: The same seed gives the same answer
      Given seat 1 holds "A♠ A♥"
      And seat 2 holds unknown cards
      And sampling is seeded with 7 and capped at 5000 samples
      When the equity is calculated
      And the equity is calculated again
      Then both calculations agree exactly

    Scenario: Aces against a full ring of unknown hands
      Given seat 1 holds "A♠ A♥"
      And 5 more seats hold unknown cards
      And sampling is seeded with 1 and capped at 20000 samples
      When the equity is calculated
      Then the answer is a Monte Carlo estimate
      And seat 1 has between 45.0% and 53.0% equity
      And the seats' equity adds up to 100%

  Rule: Impossible deals are refused

    Scenario: The same card in two hands
      Given seat 1 holds "A♠ K♠"
      And seat 2 holds "A♠ Q♠"
      When the equity is calculated
      Then the calculation is refused

    Scenario: A hole card that is also on the board
      Given seat 1 holds "A♠ K♠"
      And seat 2 holds "Q♥ Q♣"
      And the board is "A♠ 7♦ 2♣"
      When the equity is calculated
      Then the calculation is refused

    Scenario: A lone player has no one to beat
      Given seat 1 holds "A♠ K♠"
      When the equity is calculated
      Then the calculation is refused

    Scenario: Eleven seats is one more than a table holds
      Given seat 1 holds "A♠ K♠"
      And 10 more seats hold unknown cards
      When the equity is calculated
      Then the calculation is refused
