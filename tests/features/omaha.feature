@omaha
Feature: Omaha hands use exactly two hole cards
  An Omaha player holds four cards and must make a hand from exactly two of
  them plus exactly three from the board. Hands that would be obvious in
  hold'em are often not there at all.

  Scenario: One spade in hand is not a flush, however many are on the board
    When the Omaha hand "A♠ K♥ Q♥ 3♣" plays the board "2♠ 5♠ 8♠ J♠ K♦"
    Then the hand is a "pair"
    And its class is "pair of kings"

  Scenario: The same cards in hold'em make the nut flush
    When I evaluate the hand "A♠ K♥ 2♠ 5♠ 8♠ J♠ K♦"
    Then the hand is a "flush"

  Scenario: A straight on the board is not a straight in hand
    When the Omaha hand "2♣ 3♦ 7♥ 8♣" plays the board "T♠ J♦ Q♣ K♥ A♠"
    Then the hand is a "high card"

  Scenario: Three aces in hand still play only two
    When the Omaha hand "A♠ A♥ A♦ 2♣" plays the board "A♣ K♠ K♥ 7♦ 3♦"
    Then the hand is a "full house"
    And its class is "aces over kings"
    And the best five cards are "A♠ A♥ A♣ K♠ K♥"

  Scenario: Two hole cards complete a straight with three from the board
    When the Omaha hand "9♣ 8♦ 2♥ 2♠" plays the board "T♠ J♦ Q♣ 4♥ 3♠"
    Then the hand is a "straight"
    And its class is "queen high straight"
