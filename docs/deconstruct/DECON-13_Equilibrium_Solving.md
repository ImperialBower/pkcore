# DECON-13: Equilibrium Solving

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

Poker is a game of hidden information, so there is no "best move" in the sense
a chess engine means it — only a strategy that cannot be profitably exploited.
This epic specifies how a rebuild finds one: **counterfactual-regret
minimisation**, run over a **game tree** whose decision points are grouped into
**information sets**, producing an **average strategy** whose distance from
equilibrium is measurable as **exploitability**.

The honest problem with specifying a solver is that solver output is hard to
check. A number that came out of a hundred thousand iterations over a river
subtree can be plausible and wrong, and nothing in the number says which. This
is why the epic is built around a **toy-game oracle**. Kuhn poker — three cards,
two players, one betting round — has an equilibrium that is known *analytically*,
in closed form, including exact mixed frequencies and an exact game value. A
solver that reproduces it is demonstrably correct on the mechanism that matters:
regret accumulation, information-set grouping, strategy averaging, and
best-response measurement. A solver that merely produces plausible river
frequencies has demonstrated nothing.

That asymmetry mirrors the original honestly, and a rebuilder should know it
going in. The toy game in the source is densely tested — dozens of tests over
its rules, its tree, its analytic strategy family, and its solver, plus a
standalone reimplementation whose tests pin every uniquely-determined
equilibrium frequency. The general solver is not tested that way. Its tests are
structural and directional — that iteration counters advance, that frequencies
sum to one, that exploitability is non-negative and decreases, that one variant
converges faster than another, that results survive a round trip — and **not
one of them checks a strategy value against a known-correct answer.** The toy
game is therefore the trustworthy ground truth in this slice; the general
solver's behaviour is weakly verified in the original, and a rebuild should
treat the general path as specified-but-unwitnessed and lean on the oracle.

## Status

| Component | Status |
|---|---|
| Game-tree construction (decision, chance, terminal nodes) | Planned |
| Information-set grouping | Planned |
| Regret accumulation and regret matching | Planned |
| Strategy averaging over iterations | Planned |
| Exploitability measurement by best response | Planned |
| The three algorithm variants | Planned |
| Bet sizing as exact pot fractions | Planned |
| Subgame solve configuration and stopping rules | Planned |
| Toy-game rules, tree, and payoffs | Planned |
| Toy-game analytic equilibrium family | Planned |
| Toy-game solver convergence against the oracle | Planned |

## Goals

- Build a **game tree** of decision nodes, chance nodes, and terminal nodes with
  payoffs, for a river or turn subgame given ranges, a board, an effective
  stack, and a pot.
- Group decision points into **information sets** and define strategy per
  information set, never per node.
- Accumulate **regret** and derive each iteration's strategy by **regret
  matching**.
- Average strategies across iterations, and converge the **average** — not the
  final — strategy to equilibrium.
- Measure **exploitability** as the best response's advantage, and drive it
  toward zero.
- Offer three **algorithm variants** as convergence-rate choices with identical
  equilibrium guarantees.
- Reproduce the **toy-game analytic equilibrium** exactly, as the correctness
  oracle for all of the above.

## Scope

### The general solver

**A game tree** has three kinds of node. A **decision node** names the player to
act, the pot at that point, an ordered list of available actions, and one child
per action. A **chance node** names the street being dealt and carries one child
per card that can legally appear. A **terminal node** names how the hand ended —
one player folded, or both reached showdown — the final pot, and, for
multi-street trees, the specific runout card that led to it.

**Terminal valuation** is zero-sum and expressed from one player's side. A fold
pays the non-folding player half the pot at that node and costs the folder the
same. A showdown pays half the pot to the stronger hand and costs the weaker the
same; a tie pays zero to both. The winner of a showdown is determined by the
ranking rules of DECON-02.

**An information set** is a set of game states one player cannot tell apart:
the same private cards and the same publicly observable history, differing only
in what the opponent holds. Strategy is defined **per information set**, because
a player who cannot distinguish two states cannot play them differently. This is
not an optimisation — it is the defining constraint of imperfect-information
play, and a rebuild that keys strategy by node rather than by information set
produces a strategy nobody could actually execute.

**Regret matching.** Each information set carries one accumulated regret per
action. The strategy for the current iteration assigns each action a probability
proportional to its positive accumulated regret; if no accumulated regret is
positive, the strategy is uniform over the legal actions.

**Regret accumulation.** At an information set, for each action, the regret
increment is the counterfactual value of always taking that action minus the
value of playing the current mixed strategy, weighted by the *opponent's* reach
probability into that information set. Weighting by the opponent's reach is what
makes the procedure valid under hidden information.

**Strategy averaging.** Each iteration adds the current strategy, weighted by
the acting player's own reach probability, into a running strategy sum. The
**average strategy** is that sum normalised per information set, defaulting to
uniform when an information set has never been reached. It is the average
strategy — not the current one — that converges to equilibrium. A rebuild that
reports the last iteration's strategy is reporting a value that does not
converge at all.

**Exploitability** is the amount a best-responding opponent gains against the
average strategy, aggregated over both players. It is non-negative, it is zero
at equilibrium, and it decreases as training proceeds. A best response respects
the same information-set constraint as any other strategy: the best responder
conditions only on its own cards and the public history, never on the opponent's
hidden cards.

**The three variants** differ only in how updates are weighted, never in the
equilibrium they approach:

| Variant | Regret handling | Strategy weighting |
|---|---|---|
| Vanilla | Accumulate, floored at zero | Uniform across iterations |
| Plus | Accumulate, floored at zero | Linear in the iteration index |
| Discounted | Multiply accumulated regret by `t^α / (t^α + 1)` before each update | Multiply the strategy sum by `t^β / (t^β + 1)` before each update |

Discounting with `β = 0` yields a constant one-half factor each iteration,
which is equivalent to linear strategy weighting. The default variant is
discounted with `α = 1.5` and `β = 0`.

**Bet sizing** is expressed as an exact rational fraction of the pot, so that
chip amounts stay whole: `chips = pot × numerator ÷ denominator`. A denominator
of zero is an error. Named sizes cover one third, one half, two thirds, three
quarters, one, one and a half, and two times the pot. Sizings are configured per
street.

**A solve** is configured with two ranges, a board of three to five cards, an
effective stack, a starting pot, the per-street bet sizings, a maximum iteration
count, a target exploitability, and a variant. It runs until either the
iteration cap is reached or exploitability falls to the target, and returns the
average strategy together with its exploitability. Defaults: ten thousand
iterations and a target exploitability of one tenth of a chip.

### The toy-game oracle

**Rules.** A three-card deck — jack, queen, king, ordered jack below queen below
king. Two players. Each antes one chip, so the pot starts at two. Each player
receives one card; there are no community cards. Six deals are possible — the
ordered pairs of distinct cards — and all are equally likely.

The first player acts first and may **check** or **bet** one chip. After a
check, the second player may check, ending the hand at showdown, or bet one
chip, after which the first player may fold or call. After a bet, the second
player may fold or call. At showdown the higher card wins the pot.

**Tree shape.** There are four decision points — the first player's opening
decision, the second player's decision after a check, the second player's
decision facing a bet, and the first player's decision facing a check-then-bet —
and three possible private cards, giving **twelve information sets**. There are
**five terminal action sequences**: check-check, bet-fold, bet-call,
check-bet-fold, and check-bet-call. Across the six deals that is thirty terminal
leaves.

*Divergence note:* the source's own prose describes the tree as having "12
terminal nodes". The code shows twelve *information sets* and five terminal
action sequences. The counts above follow the code.

**Payoffs**, stated for the first player; the second player's are the negation:

| Terminal sequence | Pot | First player's payoff |
|---|---|---|
| check, check | 2 | +1 with the higher card, −1 otherwise |
| bet, fold | 2 | +1 |
| bet, call | 4 | +2 with the higher card, −2 otherwise |
| check, bet, fold | 2 | −1 |
| check, bet, call | 4 | +2 with the higher card, −2 otherwise |

**Legality.** From an empty history or after a single check, the legal actions
are check and bet. Facing a bet — whether the opening bet or the bet after a
check — the legal actions are fold and call. A terminal state has no legal
actions, has no player to act, and yields a payoff; a non-terminal state has no
payoff. Dealing the same card to both players is an error, and applying an
illegal action is an error.

**An information set** in this game is exactly the acting player's card plus the
public action history.

**The analytic equilibrium.** The equilibrium is a **family**, parameterised by
a single value between zero and one third inclusive. Writing that parameter as
`a`:

| Situation | Action | Frequency |
|---|---|---|
| First player, opening, jack | bet | `a` |
| First player, opening, queen | bet | 0 |
| First player, opening, king | bet | `3a` |
| Second player after a check, jack | bet | 1/3 |
| Second player after a check, queen | bet | 0 |
| Second player after a check, king | bet | 1 |
| Second player facing a bet, jack | call | 0 |
| Second player facing a bet, queen | call | 1/3 |
| Second player facing a bet, king | call | 1 |
| First player facing check-then-bet, jack | call | 0 |
| First player facing check-then-bet, queen | call | `a + 1/3` |
| First player facing check-then-bet, king | call | 1 |

The **game value to the first player is exactly −1/18**, for every member of the
family. Within the family the first player's king bet rate is exactly three
times its jack bluff rate. The parameter must lie in `[0, 1/3]`; anything
outside is an error.

The frequencies that are **uniquely determined across the whole family** — and
therefore the ones a converged solver must reproduce regardless of which member
it lands on — are: the first player never bets a queen; the first player always
calls with a king and always folds a jack facing a bet; the second player always
calls a king and never calls a jack; the second player always bets a king after
a check; the second player bluffs a jack after a check exactly one third of the
time; and the second player calls with a queen exactly one third of the time.

*Divergence note:* the source's standalone solver test module lists the **first**
player's queen-call frequency as uniquely one third. That is inconsistent with
the in-tree analytic strategy, which defines it as `a + 1/3`, and equals one
third only at `a = 0`. The test asserting one third is disabled in the source
and annotated as needing over a million iterations to pass. Spec the analytic
form: `a + 1/3`. It is not a uniquely-determined frequency and must not be
asserted as one.

**Solving the toy game.** Each iteration traverses all six deals exactly — the
tree is small enough that chance is enumerated rather than sampled, which makes
the procedure deterministic. After sufficient iterations the average strategy
matches the uniquely-determined frequencies and the exploitability of the
average strategy approaches zero. Exploitability here is computed by enumerating
every pure policy available to the best responder — one binary choice at each of
its six information sets, so sixty-four policies — evaluating each across all
six deals, and taking the maximum; the two players' best-response gains are
then aggregated.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Toy-game rules | Three cards, two players, ante one each, one betting round | `kuhn-tree.json` |
| Legal actions | Check/bet from an empty or single-check history; fold/call facing a bet; none at terminal | `kuhn-tree.json` |
| Action application | Legal actions advance the state; illegal ones are errors; duplicate deals are errors | `kuhn-tree.json` |
| Terminal detection and payoffs | Five terminal sequences with the payoff table above | `kuhn-tree.json` |
| Information sets | Twelve; acting player's card plus public history | `kuhn-tree.json`, `kuhn-equilibrium.json` |
| Analytic equilibrium family | Parameterised by `a` in `[0, 1/3]`; the frequency table above | `kuhn-equilibrium.json` |
| Game value | Exactly −1/18 to the first player, for every member of the family | `kuhn-equilibrium.json` |
| Uniquely-determined frequencies | The nine pinned entries, reproduced by a converged solver | `kuhn-equilibrium.json` |
| Regret matching | Probability proportional to positive regret; uniform when none is positive | `kuhn-equilibrium.json` |
| Strategy averaging | Reach-weighted sum, normalised; uniform for unvisited sets | `kuhn-equilibrium.json` |
| Exploitability | Non-negative, zero at equilibrium, decreasing with training | `kuhn-equilibrium.json` |
| Algorithm variants | Same equilibrium, differing convergence rates | `kuhn-equilibrium.json` |
| Bet sizing | Exact pot fractions in whole chips; zero denominator is an error | — |
| Subgame configuration | Ranges, board, effective stack, pot, sizings, caps, target, variant | — |

## Design

### Why information sets, and not nodes

Two states in which a player holds the same cards and has seen the same public
history are, to that player, the same situation. A strategy that assigned them
different action probabilities would require the player to know something it
cannot know. Grouping states into information sets is therefore not a data
structure choice — it is the encoding of the imperfect-information constraint,
and every part of the algorithm inherits it: regret accumulates per information
set, strategy is derived per information set, the average is normalised per
information set, and a best response chooses one action per information set
rather than one per node.

### The iteration

```
for each iteration:
  for each deal:
    traverse(root, reach_first = 1, reach_second = 1)

traverse(state, reach_first, reach_second):
  if state is terminal:            return payoff
  I       = information set of the acting player at `state`
  sigma   = regret_match(regret[I])        # positive-regret proportional, else uniform
  for each legal action a:
    child_value[a] = traverse(apply(state, a), reach updated for the actor)
  node_value = sum over a of sigma[a] * child_value[a]
  for each a:
    regret[I][a]       += opponent_reach * (child_value[a] − node_value)
    strategy_sum[I][a] += own_reach * sigma[a]
  return node_value
```

The two reach probabilities do different jobs and must not be interchanged.
Regret is weighted by the *opponent's* reach, because the counterfactual asks
"had I always played this action, how would I have done, given the opponent
played as it did to get here". The strategy sum is weighted by the actor's *own*
reach, because the average strategy must be weighted by how often the actor
actually arrives at that information set.

Values are carried in a single consistent frame throughout — either always the
first player's, with the sign of the regret expression flipped for the second
player, or always the acting player's, with each child's value negated on
return. Either is correct; mixing them is not.

### Regret flooring and the variants

Flooring accumulated regret at zero after each update discards stale negative
regret, so the current strategy tracks the opponent's present behaviour instead
of its early-iteration behaviour. This is the "plus" idea. The plus variant pairs
it with linear strategy weighting, so chaotic early iterations contribute less to
the average. The discounted variant generalises both: a regret discount of
`t^α / (t^α + 1)` and a strategy discount of `t^β / (t^β + 1)` applied before
each update.

All three converge to the same equilibrium. The choice is a convergence-rate
choice and must be observable only as *how fast* exploitability falls, never as
*where* it falls to. A rebuild must be able to demonstrate that: the same
subgame solved under each variant to a low enough exploitability must agree on
the resulting frequencies.

### Exploitability

Exploitability is the measurable distance from equilibrium. For each player,
compute the value of that player's best response against the opponent's average
strategy; the aggregate of the two gains is the exploitability of the profile.
It is non-negative because a best responder never does worse than equilibrium
play, and it is bounded above by the largest swing the pot permits.

The best response must be computed under the information-set constraint. For the
toy game this is direct: the best responder has six information sets, each with
two actions, so all sixty-four pure policies can be enumerated and the best
selected. For a general subgame the same principle applies — the best responder
maximises at its own decision points while the opponent follows the fixed
average strategy — but the enumeration is replaced by a traversal.

The exact normalisation is a reporting convention rather than a domain fact; see
*Not specified*. What is normative is that the figure is non-negative, is zero
at equilibrium, and decreases monotonically in expectation with training.

### The oracle, and how to use it

The toy game exists in this spec for one reason: it converts "my solver produces
plausible numbers" into "my solver produces the known-correct numbers".

Build it first. Its rules, tree, and payoffs are small enough to state
completely and pin exactly — that is what `kuhn-tree.json` contains, and it is
checkable without any solver at all. Then implement the analytic strategy family
directly from the frequency table, which gives a reference strategy at any
parameter value with no iteration involved. Only then run the solver on it, and
require the average strategy to converge to the uniquely-determined frequencies
and the game value to −1/18.

This ordering matters. If the solver disagrees with the analytic strategy, the
solver is wrong — there is no ambiguity to argue about, no tolerance to widen,
no "well, it's a different equilibrium". The one genuine degree of freedom is
the family parameter, and even that is constrained: whichever member the solver
lands on, the first player's king bet rate must be exactly three times its jack
bluff rate, and the nine pinned frequencies must hold regardless.

Reproducing this analytic solution is far stronger evidence of correctness than
reproducing any sampled number from a large tree, because it tests the mechanism
against a closed-form answer rather than against another implementation's
output.

### Convergence tolerance

Different information sets converge at very different rates. Dominated actions
resolve quickly. Genuinely mixed frequencies converge more slowly. Information
sets at a point of *exact indifference* — where both actions have equal value at
equilibrium — converge slowest of all, because the average strategy there is
driven by the ratio of accumulated regrets, which early iterations dominate. The
source records one such information set requiring on the order of a million
iterations to settle.

A rebuild must therefore state the iteration count alongside every tolerance it
claims, and must not treat a slow-converging information set as a defect. The
vector file carries the iteration counts and tolerances its values were measured
at.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Change the payoff structure, the deck, or what beats what | The equilibrium is a consequence of the rules, never an input to the solve |
| **Administrative** | Configure ranges, board, stack, pot, sizings, iteration cap, target exploitability, and variant | Configure the answer — no setting changes which strategies are in equilibrium | An operator chooses how hard and how the search runs, never where it must land |
| **User/client** | Request a solve and read the resulting strategy and its exploitability | Obtain a strategy conditioned on cards the acting player cannot see | Every returned strategy is executable by a player with only that player's information |
| **Observer/operator** | Read iteration count and exploitability from a completed solve | — | A finished solve reports how far from equilibrium it got; a running solve reports nothing — there is no progress signal and no cancellation |
| **Agent** | Consult a solved strategy for its own information set | Consult an information set that is not its own | An agent reads only the row of the strategy that corresponds to what it knows |
| **Trainer/researcher** | Rerun any solve to the same result; measure any strategy's exploitability without altering it; validate against the toy-game oracle | — | Any experiment is repeatable, and correctness is checkable against a closed-form answer rather than against another run |
| **Spectator** | N/A — no delivery surface in this slice. | | |
| **Trustless/cryptographic peer** | N/A — no commitments or verifiable computation in this slice. | | |

*Performant (informative, per SD-08):* the toy game enumerates chance exactly
rather than sampling it, which makes every iteration deterministic; the variant
choice is the exposed convergence-rate lever. Counter-observation: no
equilibrium-solving benchmark or regression gate exists in the original.

## Work Items

### Phase 0 — The toy game's rules and tree

- [ ] **0a.** Stand up a runner over `vectors/equilibrium-solving/kuhn-tree.json`
  and `kuhn-equilibrium.json`.
- [ ] **0b.** Write failing tests for card ordering, deal validity (identical
  cards rejected), and the six possible deals.
- [ ] **0c.** Write failing tests for legal actions at each of the four decision
  points, for terminal detection over the five terminal sequences, and for the
  errors: applying an illegal action, and asking a non-terminal state for a
  payoff.
- [ ] **0d.** Implement the state, its transitions, and terminal detection until
  every node and edge in `kuhn-tree.json` is reproduced.
- [ ] **0e.** Write failing tests for the payoff table — all five sequences under
  both card orderings — then implement payoffs against `kuhn-tree.json`.
- [ ] **0f.** Implement information-set derivation and prove there are exactly
  twelve, matching `kuhn-tree.json`.

### Phase 1 — The analytic equilibrium

- [ ] **1a.** Write failing tests that the parameter is accepted on `[0, 1/3]`
  and rejected outside it.
- [ ] **1b.** Write failing tests for the full frequency table at three
  parameter values, including the endpoints, asserting each information set's
  probabilities sum to one.
- [ ] **1c.** Implement the analytic strategy family and reproduce
  `kuhn-equilibrium.json`'s reference-strategy section.
- [ ] **1d.** Write a failing test that the game value to the first player is
  −1/18 at every tested parameter value; make it pass.

### Phase 2 — Regret minimisation on the toy game

- [ ] **2a.** Write failing tests for regret matching in isolation: proportional
  to positive regret, uniform when no regret is positive.
- [ ] **2b.** Write failing tests for strategy averaging: uniform before any
  training, reach-weighted normalised sum after.
- [ ] **2c.** Implement the traversal with correct reach weighting — opponent's
  reach on regret, own reach on the strategy sum — and full enumeration of the
  six deals per iteration.
- [ ] **2d.** Write failing tests that the average strategy reproduces each of
  the nine uniquely-determined frequencies within the tolerance and iteration
  count stated in `kuhn-equilibrium.json`.
- [ ] **2e.** Write a failing test that the first player's king bet rate is three
  times its jack bluff rate, whatever member of the family the solver lands on.
- [ ] **2f.** Write a failing test that the solved game value converges to −1/18.

### Phase 3 — Exploitability

- [ ] **3a.** Write failing tests that exploitability is strictly positive before
  training and falls below the threshold stated in `kuhn-equilibrium.json` after
  the stated iteration count.
- [ ] **3b.** Write a failing test that a best response conditions only on its
  own card and the public history.
- [ ] **3c.** Implement best-response computation and exploitability; verify the
  exploitability of the analytic strategy is at or near zero — a direct check of
  the measurement itself, independent of the solver.

### Phase 4 — The variants

- [ ] **4a.** Write failing tests that each of the three variants runs and drives
  exploitability down on the toy game.
- [ ] **4b.** Write a failing test that all three, run to a low exploitability,
  agree on the uniquely-determined frequencies — the variants are rate choices,
  not answer choices.
- [ ] **4c.** Implement regret flooring, linear strategy weighting, and the two
  discount factors, and prove the discount formulas at specific iteration
  indices.

### Phase 5 — The general subgame solver

- [ ] **5a.** Write failing tests for bet sizing: exact pot fractions in whole
  chips, the seven named sizes, and rejection of a zero denominator.
- [ ] **5b.** Write failing tests for tree construction on a river subgame:
  decision nodes carry matching action and child counts, terminal nodes carry
  the correct pot, and a fold terminal pays half the pot to the non-folder.
- [ ] **5c.** Write failing tests for turn tree construction: a chance node
  carries one child per legal runout card, and its terminals record which card
  led to them.
- [ ] **5d.** Implement tree construction, the solve loop with both stopping
  rules, and the returned average strategy and exploitability.
- [ ] **5e.** Write failing tests that per-information-set frequencies sum to
  one, that exploitability is non-negative and bounded by half the pot, and that
  it decreases with more iterations.
- [ ] **5f.** Close the loop: express the toy game as a subgame configuration for
  the general solver where the shapes allow, and require the general path to
  reproduce the oracle. This is the work item that upgrades the general solver
  from structurally-tested to correctness-tested.

## Test Plan

**Tree shape.**
*Given* the toy game's rules, *when* the tree is enumerated, *then* it matches
`kuhn-tree.json`: four decision points, twelve information sets, five terminal
action sequences, thirty leaves across six deals.

**Legality and errors.**
*Given* each state in `kuhn-tree.json`, *when* legal actions are requested,
*then* they match the file; *and when* an action outside that set is applied,
*then* it is rejected; *and when* a non-terminal state is asked for a payoff,
*then* it is rejected; *and when* both players are dealt the same card, *then*
it is rejected.

**Payoffs.**
*Given* each terminal in `kuhn-tree.json`, *when* the payoff is computed, *then*
it matches the file exactly for both players, and the two sum to zero.

**Analytic frequencies.**
*Given* each parameter value in `kuhn-equilibrium.json`, *when* the analytic
strategy is built, *then* every information set's probabilities match the file
and sum to one; *and when* the parameter is outside `[0, 1/3]`, *then* it is
rejected.

**Game value.**
*Given* the analytic strategy at any parameter value, *when* the value to the
first player is computed over all six deals, *then* it is −1/18 within the
file's tolerance.

**Regret matching.**
*Given* an information set with mixed-sign accumulated regret, *when* the
current strategy is derived, *then* probabilities are proportional to the
positive regrets and zero elsewhere; *and given* one with no positive regret,
*then* the strategy is uniform.

**Averaging converges, the current strategy need not.**
*Given* a trained solver, *when* the average strategy is read, *then* it matches
the pinned frequencies in `kuhn-equilibrium.json`; the final iteration's
strategy carries no such guarantee and is not asserted.

**Pinned frequencies.**
*Given* the iteration count in `kuhn-equilibrium.json`, *when* training
completes, *then* each of the nine uniquely-determined frequencies is within the
stated tolerance, and the first player's king bet rate is three times its jack
bluff rate.

**Exploitability falls.**
*Given* an untrained solver, *then* exploitability is strictly positive; *when*
trained for the counts in `kuhn-equilibrium.json`, *then* it falls below each
stated threshold, and it is never negative.

**Exploitability of a known equilibrium.**
*Given* the analytic strategy rather than a solved one, *when* exploitability is
measured, *then* it is at or near zero — validating the measurement itself.

**Variants agree.**
*Given* the same problem solved under each of the three variants to a low
exploitability, *then* all three agree on the uniquely-determined frequencies
within the file's tolerance, while differing in the iterations required.

**Bet sizing.**
*Given* a pot and a named or custom fraction, *when* chips are computed, *then*
the result is the exact whole-chip product-then-quotient; *and given* a zero
denominator, *then* it is rejected.

**Subgame terminals.**
*Given* a river subgame, *when* a fold terminal is valued, *then* the non-folder
receives half that node's pot; *and when* a showdown terminal is valued, *then*
the stronger hand receives half the pot and a tie pays zero.

## Not specified (implementer's choice)

- **Tree representation.** An explicit materialised tree, a lazily expanded one,
  or a purely recursive traversal that never stores nodes are all acceptable.
  Only the enumerated shape in `kuhn-tree.json` is binding.
- **Information-set keying.** How an information set is identified — a composed
  key, a numeric index, a hash — is free; only the grouping it induces is
  normative.
- **The value frame.** Carrying values always in the first player's frame with a
  sign flip in the regret expression, or always in the acting player's frame with
  a negation on return, are equivalent.
- **Exploitability normalisation.** The sum of both players' best-response gains
  and half their difference are both in use in the original. Any convention is
  acceptable provided it is documented, non-negative, and zero at equilibrium.
  Vectors state which convention their figures use.
- **Best-response computation.** Exhaustive pure-policy enumeration and
  traversal-based maximisation are equivalent; the toy game admits both.
- **Iteration scheduling.** Simultaneous or alternating updates, serial or
  parallel traversal — free, so long as the converged result is unchanged.
- **Chance handling in the general solver.** Full enumeration or sampling of
  runouts, provided the stated exploitability is honest about which was used.
- **Numeric representation and tolerances** beyond those stated in the vectors.
- **Persistence of a solved strategy.** Whether and how results are stored is
  out of scope pack-wide.
- **Which member of the equilibrium family a solver lands on.** Any value of the
  parameter in `[0, 1/3]` is a correct answer, provided the pinned frequencies
  and the three-times relation hold.

## Spec decisions

None. The exploitability normalisation and the family parameter are recorded
above as named freedoms rather than as decisions requiring a pinned choice.

## Verification

Any implementation must reproduce every file under
`vectors/equilibrium-solving/`:

1. `kuhn-tree.json` is reproduced exactly: every state, its legal actions, its
   acting player, its information set, its terminal status, and every terminal's
   payoff for both players.
2. The tree contains exactly twelve information sets and five terminal action
   sequences, and thirty terminal leaves across the six equally-likely deals.
3. `kuhn-equilibrium.json`'s reference strategies are reproduced at every
   parameter value it lists, with each information set's probabilities summing
   to one, and parameters outside `[0, 1/3]` rejected.
4. The game value to the first player is −1/18 within the file's tolerance, for
   every parameter value tested.
5. A solver trained for the iteration counts in `kuhn-equilibrium.json`
   reproduces all nine uniquely-determined frequencies within the stated
   tolerances.
6. The solved first player's king bet rate is three times its jack bluff rate
   within tolerance, whichever member of the family was reached.
7. Exploitability is strictly positive before training, non-negative always,
   falls below each threshold stated in `kuhn-equilibrium.json` at the stated
   iteration count, and is at or near zero when measured against the analytic
   strategy directly.
8. All three algorithm variants reach the same uniquely-determined frequencies
   within tolerance, differing only in the iterations required.
9. Bet sizes produce exact whole-chip amounts for the seven named fractions and
   reject a zero denominator.
10. A river subgame's fold terminals pay half the node's pot to the non-folder,
    its showdown terminals pay half the pot to the stronger hand and zero on a
    tie, and a turn subgame's chance node carries one child per legal runout
    card.
11. Every reported strategy is executable from its own information set alone:
    no returned frequency is conditioned on the opponent's hidden cards.

## Dependencies

**Builds on:** DECON-04 (Range Notation) for the ranges a subgame is solved
over; DECON-09 (Equity and Odds) for the showdown accounting underlying terminal
valuation; DECON-02 (High Hand Ranking) for the comparison at showdown.

**Blocks:** nothing in this pack. This is the terminal epic in the build order.

## Provenance (non-normative)

- `src/games/kuhn.rs:1` — toy-game rules, ante, betting round, and showdown.
- `src/games/kuhn.rs:50` — the three-card ordering.
- `src/games/kuhn.rs:83` — the four actions.
- `src/games/kuhn.rs:302` — the game state as an immutable value; `:325`
  rejects a duplicate deal.
- `src/games/kuhn.rs:383` — terminal detection over the five sequences.
- `src/games/kuhn.rs:417` — which player acts, by history length.
- `src/games/kuhn.rs:439` — legal actions per history.
- `src/games/kuhn.rs:465` — action application and rejection of illegal actions.
- `src/games/kuhn.rs:510` — the payoff table and its pot column.
- `src/games/kuhn.rs:555` — information set as card plus public history.
- `src/games/kuhn.rs:600` — the analytic frequency table, including the
  first player's queen call at `a + 1/3`.
- `src/games/kuhn.rs:630` — parameter validation on `[0, 1/3]`.
- `src/games/kuhn.rs:667` — construction of the twelve-entry strategy table.
- `src/games/kuhn.rs:752` — the default parameter of one third and the −1/18
  game value.
- `src/games/kuhn.rs:760` — the six equally-likely deals.
- `src/games/kuhn.rs:789` — the solver; `:840` full enumeration of all six deals
  per iteration.
- `src/games/kuhn.rs:868` — average strategy, uniform for unvisited sets.
- `src/games/kuhn.rs:923` — exploitability as the aggregate of both
  best-response values.
- `src/games/kuhn.rs:936` — the traversal, its reach weighting, and the value
  frame; `:975` opponent's reach on regret and own reach on the strategy sum.
- `src/games/kuhn.rs:993` — regret matching, uniform when nothing is positive.
- `src/games/kuhn.rs:1036` — best response by enumerating all sixty-four pure
  policies under the information-set constraint.
- `tests/kuhn_poker.rs:28` — the equilibrium family and the uniquely-determined
  frequencies.
- `tests/kuhn_poker.rs:54` — the three-times relation and the −1/18 value.
- `tests/kuhn_poker.rs:60` — regret matching, counterfactual weighting, and
  averaging described in prose.
- `tests/kuhn_poker.rs:190` — regret flooring at zero.
- `tests/kuhn_poker.rs:241` onward — the pinned frequency tests.
- `tests/kuhn_poker.rs:333` — records that the first player's queen call does
  not converge to one third; the corresponding test is disabled.
- `src/analysis/gto/solver_config.rs:51` — the three variants and their
  discount factors; `:77` the discounted default.
- `src/analysis/gto/solver_config.rs:106` — bet size as an exact rational
  fraction; `:254` whole-chip computation.
- `src/analysis/gto/solver_config.rs:304` — per-street sizings.
- `src/analysis/gto/solver_config.rs:388` — the solve configuration; `:433` the
  ten-thousand-iteration and one-tenth-chip defaults.
- `src/analysis/gto/game_tree.rs:133` — the two positions.
- `src/analysis/gto/game_tree.rs:182` — actions carrying their sizing.
- `src/analysis/gto/game_tree.rs:218` — terminal outcomes: fold or showdown.
- `src/analysis/gto/game_tree.rs:248` — decision nodes with matching action and
  child lists.
- `src/analysis/gto/game_tree.rs:259` — chance nodes with one child per runout
  card.
- `src/analysis/gto/game_tree.rs:280` — terminal nodes carrying the runout card
  for multi-street trees.
- `src/analysis/gto/regret.rs:88` — accumulated regret per node and holding.
- `src/analysis/gto/regret.rs:169` — regret flooring at zero.
- `src/analysis/gto/regret.rs:218` — regret matching, uniform fallback.
- `src/analysis/gto/regret.rs:304` — scaling all regrets, used for discounting.
- `src/analysis/gto/strategy_profile.rs:63` — normalised action frequencies.
- `src/analysis/gto/strategy_profile.rs:301` — the uniform starting profile.
- `src/analysis/gto/solver.rs:730` — one iteration; `:830` the solve loop and
  its stopping rules; `:875` the averaged equilibrium strategy.
- `src/analysis/gto/solver.rs:1026` — exploitability as half the gap between the
  two best-response values.
- `src/analysis/gto/solver.rs:1314` — terminal valuation: half the pot on a
  fold, half the pot to the stronger hand at showdown, zero on a tie.
- `src/analysis/gto/solver.rs:1345` — best response maximising at its own
  decision points against a fixed opponent strategy.
- `src/analysis/gto/solver.rs:1538` onward — the general solver's tests: all
  structural or directional, none checking a strategy value against a
  known-correct answer.
