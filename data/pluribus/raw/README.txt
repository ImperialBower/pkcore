This contains the hands played in the 5 humans + 1 AI set of experiments involving Pluribus. This document describes the format of this data.

Each file contains a series of lines that look like this:

STATE:7:fr225fffc/cr475c/cr1225c/cc:5hJc|Jd9h|6s5c|Ah7h|2s2d|3hTs/3sJh2h/Tc/Ks:-50|1275|0|-1225|0|0:Gogo|Budd|Eddie|Bill|Pluribus|MrWhite

STATE means this line is a hand of poker

7 is the index of the hand in this session for this table

fr225fffc/cr475c/cr1225c/cc is the sequence of actions that have occurred in this hand. '/' signifies the end of a betting round, 'f' means fold, 'c' means call, 'r' means raise. The number following an 'r' is the total number of chips that player has in the pot after the raise (including money from all prior betting rounds in this hand).

5hJc|Jd9h|6s5c|Ah7h|2s2d|3hTs/3sJh2h/Tc/Ks shows the cards that were dealt to the players and the public board cards that were revealed. Each card is represented by two characters: the first showing the rank and the second showing the suit. 2 through 9 are possible ranks of the cards. Additionally, T = 10, J = Jack, Q = Queen, K = King, and A = Ace. 's', 'h', 'd', 'c' are the possible suits of the cards. The private cards dealt to the players are shown as pairs of cards with '|' separating the players. The '/' signifies the start of the public board cards. The first three cards after the first '/' are the flop cards (revealed on the second betting round). The card after the next '/' is the turn card (revealed on the third betting round). The card after the last '/' is the river card (revealed on the fourth betting round). If the game ends before a board card is revealed, that board card will not be listed.

-50|1275|0|-1225|0|0 shows the amount of money won or lost by each player, with each player separated by a '|'

Gogo|Budd|Eddie|Bill|Pluribus|MrWhite shows the participants in this hand and their positions. In this case, Gogo is the small blind and Budd is the big blind. Eddie is the first player to act on the first betting round in this hand. On all subsequent rounds, Gogo will be the first player to act (if still in the hand).