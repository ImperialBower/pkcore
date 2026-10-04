@hands
Feature: Ranking poker hands
  pkcore ranks every five-card hand with Cactus Kev's scheme: 7,462 distinct
  values, where 1 is a royal flush and 7462 is seven-high. Lower is stronger.
  Six and seven cards are ranked by their best five.

  Scenario Outline: Each category has its familiar name
    When I evaluate the hand "<hand>"
    Then the hand is a "<name>"
    And its class is "<class>"

    Examples:
      | hand           | name            | class                  |
      | A♠ K♠ Q♠ J♠ T♠ | straight flush  | royal flush            |
      | 5♦ 4♦ 3♦ 2♦ A♦ | straight flush  | five high straight flush |
      | 9♣ 9♦ 9♥ 9♠ 2♣ | four of a kind  | four nines             |
      | K♠ K♥ K♦ 7♣ 7♦ | full house      | kings over sevens      |
      | A♥ J♥ 8♥ 6♥ 2♥ | flush           | ace high flush         |
      | T♣ 9♦ 8♥ 7♠ 6♣ | straight        | ten high straight      |
      | 5♠ 4♥ 3♦ 2♣ A♠ | straight        | five high straight     |
      | Q♣ Q♦ Q♥ 8♠ 3♣ | three of a kind | three queens           |
      | J♠ J♥ 4♦ 4♣ A♠ | two pair        | jacks and fours        |
      | 6♦ 6♥ A♣ K♠ 2♦ | pair            | pair of sixes          |
      | A♠ Q♥ 9♦ 6♣ 3♠ | high card       | ace high               |

  Scenario Outline: The boundaries of the 7,462 values
    When I evaluate the hand "<hand>"
    Then its hand rank value is <value>

    Examples:
      | hand           | value | what                   |
      | A♠ K♠ Q♠ J♠ T♠ | 1     | the best hand there is |
      | 5♥ 4♥ 3♥ 2♥ A♥ | 10    | the worst straight flush |
      | A♠ A♥ A♦ A♣ K♠ | 11    | the best four of a kind |
      | A♠ A♥ A♦ K♣ K♠ | 167   | the best full house    |
      | A♦ K♦ Q♦ J♦ 9♦ | 323   | the best flush         |
      | A♠ K♥ Q♦ J♣ T♠ | 1600  | the best straight      |
      | 5♠ 4♥ 3♦ 2♣ A♠ | 1609  | the worst straight     |
      | 7♠ 5♥ 4♦ 3♣ 2♠ | 7462  | the worst hand there is |

  Rule: Higher hands win, and suits never break a tie

    Scenario: A flush beats a straight
      Given Alice holds "2♠ 5♠ 7♠ 9♠ J♠"
      And Bob holds "A♥ K♦ Q♣ J♥ T♠"
      Then Alice beats Bob

    Scenario: The wheel is the lowest straight
      Given Alice holds "6♦ 5♣ 4♥ 3♠ 2♠"
      And Bob holds "5♦ 4♣ 3♥ 2♥ A♠"
      Then Alice beats Bob

    Scenario: The kicker decides between equal pairs
      Given Alice holds "A♠ A♥ K♦ 7♣ 2♠"
      And Bob holds "A♦ A♣ Q♠ J♥ T♦"
      Then Alice beats Bob

    Scenario: The same ranks in different suits split the pot
      Given Alice holds "A♠ K♠ Q♦ J♣ 9♥"
      And Bob holds "A♥ K♥ Q♣ J♦ 9♠"
      Then Alice and Bob split the pot

  Rule: More than five cards play their best five

    Scenario: Gus Hansen's quad fives
      The 2006 High Stakes Poker hand: 5♦ 5♣ on a board of 9♣ 6♦ 5♥ 5♠ 8♠.
      When I evaluate the hand "5♦ 5♣ 9♣ 6♦ 5♥ 5♠ 8♠"
      Then the hand is a "four of a kind"
      And the best five cards are "5♠ 5♥ 5♦ 5♣ 9♣"

    Scenario: Daniel Negreanu's sixes full on the same board
      When I evaluate the hand "6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠"
      Then the hand is a "full house"
      And its class is "sixes over fives"

    Scenario: Hole cards that do not help play the board
      When I evaluate the hand "2♣ 3♦ A♥ K♥ Q♥ J♥ T♥"
      Then the hand is a "straight flush"
      And the best five cards are "A♥ K♥ Q♥ J♥ T♥"

    Scenario: Six cards, as on the turn
      When I evaluate the hand "A♠ A♥ K♦ K♣ Q♠ 2♦"
      Then the hand is a "two pair"
      And the best five cards are "A♠ A♥ K♦ K♣ Q♠"

  Rule: Only real hands are ranked

    Scenario: A repeated card leaves too few cards for a hand
      When I evaluate the hand "A♠ A♠ K♠ Q♠ J♠"
      Then the hand cannot be evaluated

    Scenario: Four cards are not a hand
      When I evaluate the hand "A♠ K♠ Q♠ J♠"
      Then the hand cannot be evaluated
