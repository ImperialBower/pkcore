# EPIC 00c Domain, Requirements and Logic

I am a big fan of [domain-driven design](https://en.wikipedia.org/wiki/Domain-driven_design) (DDD) as a developer. The
big idea is that when you are developing a system, you focus on the actual things that make up that system. For instance,
if you are creating a role playing game like in the [Evercraft Kata](https://github.com/PuttingTheDnDInTDD/EverCraft-Kata), 
you focus on Characters, their Classes, Ability Scores, Hit Points, etc. If you're crafting a banking application, you need to 
define things like Currency, Cash Reserves, Customers, how much Cash they have, how much they owe, and things like
Interest Rates. 

Once you define the Things of your system, you need to define the rules of how they interact, or what we call 
[Business Requirements](https://en.wikipedia.org/wiki/Business_requirements). With EverCraft, that's stuff like how if a Character's
Hit Points reach zero, their dead, and how Ability Scores go from one to twenty. With our Banking system, it would
be things like how if we lend $400 to one of our customers, our reserves go down $400, their's go up the same amount, 
as well as the same amount to what they owe. _If you haven't played with it yet, I highly recommend trying out the 
EverCraft Kata. It's a lot of fun._

To give you an idea of just how complicated things can be, let's take for instance in invention of Credit Cards. When
Banks introduced them, they instantly flooded the economy with millions of Dollars out of thin air. While the Business
Logic behind how you do this is complex, but straight forward, the Business Logic behind what effect flooding a Country
with spending power out of nowhere has on a economy is a much harder problem, and one that Economists can argue about
until we've grown ourselves into extinction.

The code that actually implements the Business Requirements is called [Business Logic](https://en.wikipedia.org/wiki/Business_logic).

Now let's apply that to Poker.

French Deck = Domain
Texas Hold 'Em = Business Requirements

One of the cool things about card games, is that you can take the exact same Domain and apply a completely different
set of Business Logic. The Domain for such Poker games as Razz, Omaha Hi-Lo, and Stud all use our same French Deck of 52
cards made up of four Suites and 13 Ranks. It's also used for games like 
[Bridge](https://en.wikipedia.org/wiki/Contract_bridge) and Single Deck [Blackjack](https://en.wikipedia.org/wiki/Blackjack). 
This changes a little bit for games where you add in Wild Cards like the Joker. NOTE: _You can have Wild Cards without
adding any extra Cards though an adjustment in your Business Logic, say making Deuces Wild, or the 
[Suicide King](https://upswingpoker.com/what-is-the-suicide-king-in-card-games/)._ 

A small change to the Business Requirements of a system, can have radical effects on it's Business Logic. Let's take
something like Deuces Wild, where the rule is that any Deuce can represent any other Card. 

Say you have a hand like 9♡ 8♡ 6♡ 5♡ 2♣, which is the 7,422th best hand in poker games like Hold 'Em and Five Card 
Draw. We need to write the Business Logic to determine what the best possible hand is. Change it to a Nine and we have
a pair of Nines. Change it to an A♡ and we have an Ace high Flush. Change it to a 7♣ and we have a Straight. But, change
it to a 7♡ and we have a Nine high Straight Flush, which can only be beaten by five other Straight Flushes. 

The Business Requirement is simple: Deuces are Wild. The Business Logic is more complex: Evaluate the hand for every 
possible card to replace the 2♣ and return to me the one that's the best, in this case 9♡ 8♡ 7♡ 6♡ 5♡. That translates
to performing an evaluation 52 times, for every possible card, and returning the one that's the highest. If a Hand 
has two Deuces you would have to do that 52 * 52, or 2,704 times. Now, maybe there's a way to figure out the best hand
without doing all that work, 




