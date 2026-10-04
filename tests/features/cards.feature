@cards
Feature: Reading cards
  Every card has an index: a rank character and a suit character. pkcore reads
  the suit as a symbol (♠ ♥ ♦ ♣, filled or hollow) or as a letter in either
  case, and always writes it back as the filled symbol.

  Scenario Outline: An index names exactly one card
    When I read the card "<index>"
    Then it is the "<card>" card

    Examples: symbols, letters and both cases
      | index | card |
      | A♠    | A♠   |
      | As    | A♠   |
      | AS    | A♠   |
      | K♡    | K♥   |
      | Td    | T♦   |
      | 9♧    | 9♣   |
      | 2c    | 2♣   |
      | qd    | Q♦   |
      |  Q♥   | Q♥   |

  Scenario Outline: Anything that is not a rank followed by a suit is refused
    When I read the card "<index>"
    Then reading the card fails

    Examples:
      | index |
      |       |
      | QQ    |
      | 1♠    |
      | ♠A    |
      | A     |
      | Ax    |

  @finding
  Scenario Outline: Characters after the suit are ignored
    A single card is read from its first two characters and the rest of the
    string is dropped without complaint, so a typo or a whole hand passed
    where one card was expected still reads as a card.
    When I read the card "<index>"
    Then it is the "<card>" card

    Examples:
      | index | card |
      | AHX   | A♥   |
      | Kh Qd | K♥   |

  Scenario: A deck holds every card exactly once
    Given a fresh deck
    Then it holds 52 cards, all different

  Scenario: A ten is written T, not 10
    When I read the cards "A♠ Kh Qd J♣ 10s"
    Then reading the cards fails

  Scenario: Several cards are separated by whitespace
    When I read the cards "A♠ Kh Qd J♣ Ts"
    Then I hold 5 cards
    And they read back as "A♠ K♥ Q♦ J♣ T♠"

  Scenario: Writing a card twice holds it once
    The index is read into a set, so a repeat is absorbed rather than
    rejected. Callers that need an exact count must check it themselves —
    which is what the fixed-size hands do (see hand_ranking.feature).
    When I read the cards "A♠ A♠ K♥"
    Then I hold 2 cards
