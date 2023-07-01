# Serialization

Idea for a common format to store and retrieve hands:

for hold'em style games:
GAME_TYPE: [CARDS HELD] - BOARD

For instance, The Hand would be displayed as:
> HE: 6♠ 6♥ 5♦ 5♣ - 9♣ 6♦ 5♥ 5♠ 8♠

And for The Fold:
> HE: 5♠ 5♦ 9♠ 9♥ K♣ T♦ - 5♣ 9♦ T♥ T♣ Q♦
