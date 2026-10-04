@table
Feature: Betting a hand of no-limit hold'em
  The table engine posts the blinds, keeps track of whose turn it is, prices
  every call and raise, and refuses anything out of turn or under the
  minimum. Seats are filled from the button: the first player listed deals.

  Background:
    Given a no-limit hold'em table with blinds of 50/100 and these players:
      | name  | chips |
      | Alice | 5000  |
      | Bob   | 5000  |
      | Carol | 5000  |
    When the blinds are posted
    And the hole cards are dealt

  Scenario: The blinds go in to the left of the button
    Then Bob is the "small blind"
    And Carol is the "big blind"
    And Bob has 50 chips in front
    And Carol has 100 chips in front
    And every chip is still on the table

  Scenario: The first player after the big blind opens the action
    Then Alice is the "under the gun"
    And the action is on Alice
    And Alice needs 100 to call
    And Bob needs 50 to call
    And Carol needs 0 to call
    And the minimum raise is to 200

  Scenario: Folded to the big blind
    When Alice folds
    And Bob folds
    Then the hand is over
    When the hand ends
    Then the winnings list 1 award
    And Carol has 5050 chips
    And Bob has 4950 chips
    And Alice has 5000 chips
    And every chip is still on the table

  Scenario: A raise sets the price and the next minimum
    The minimum re-raise repeats the last raise: 300 is 200 over the big
    blind, so the next raise must reach 500.
    When Alice raises to 300
    Then the action is on Bob
    And Bob needs 250 to call
    And Carol needs 200 to call
    And the minimum raise is to 500

  Scenario: Calling a raise moves the action on
    When Alice raises to 300
    And Bob calls
    Then the action is on Carol
    And Bob has 300 chips in front
    And Bob has 4700 chips

  Rule: Out-of-turn and illegal actions change nothing

    Scenario: The big blind cannot fold before it is her turn
      When Carol tries to fold
      Then the action is refused as out of turn
      And the action is on Alice
      And the hand is not over

    Scenario: A raise must be at least the size of the big blind
      When Alice tries to raise to 150
      Then the action is refused
      And the action is on Alice
      And Alice needs 100 to call

    Scenario: The small blind cannot act ahead of the button
      When Bob tries to call
      Then the action is refused as out of turn
      And the action is on Alice
