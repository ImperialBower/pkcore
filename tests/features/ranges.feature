@ranges
Feature: Reading hand ranges
  A range is written the way players write it — "AA", "AKs", "KK+",
  "22-55", comma separated — and stands for every two-card combination it
  covers. There are 1,326 combinations in all: 6 of each pocket pair, 4 of
  each suited hand, 12 of each offsuit hand.

  Scenario Outline: A range covers a known number of combinations
    When I read the range "<range>"
    Then it covers <combos> hole-card combinations

    Examples:
      | range           | combos | because                              |
      | AA              | 6      | six ways to pair two of four aces    |
      | AKs             | 4      | one per suit                         |
      | AKo             | 12     | four times three                     |
      | AK              | 16     | suited and offsuit together          |
      | KK+             | 12     | aces and kings                       |
      | 22+             | 78     | thirteen pairs of six                |
      | 22-55           | 24     | four pairs of six                    |
      | KK+, AKs        | 16     | spaces after the comma are ignored   |
      | AA, AA          | 6      | a repeated hand is counted once      |
      | AA:0.5          | 6      | a frequency suffix does not thin the range |
      | Ax              | 192    | an ace with any other card           |

  Scenario: A suited range is all suited
    When I read the range "AKs, KQs, QJs"
    Then it covers 12 hole-card combinations
    And every combination is suited

  Scenario: An offsuit range has no suited hands in it
    When I read the range "AKo"
    Then no combination is suited

  Scenario: A pair range is all pairs
    When I read the range "TT+"
    Then it covers 30 hole-card combinations
    And every combination is a pocket pair

  Scenario Outline: Ranges that name no hand are refused
    When I read the range "<range>"
    Then reading the range fails

    Examples:
      | range |
      | A1    |
      | ZZ    |
      | AA-   |
      | AKx   |
      | QQQ   |
