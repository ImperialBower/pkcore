# EPIC: Pluribus

This module is named `Nubibus`. It means clouds or a swarm, and it's a pun on Pluribus. 

## Links

* [Superhuman AI for multiplayer poker](https://www.science.org/doi/10.1126/science.aay2400)
* [pluribus-hand-parser](https://github.com/VitamintK/pluribus-hand-parser)
* [Let's analyze Pluribus's Hands!](http://kevinwang.us/lets-analyze-pluribuss-hands/)
  * [reddit](https://www.reddit.com/r/poker/comments/cdhasb/download_all_10000_hands_that_pluribus_poker_ai/)
* [fedden / poker_ai](https://github.com/fedden/poker_ai) - An Open Source Texas Hold'em AI
* [Remembering Pluribus: The Techniques that Facebook Used to Master World’s Most Difficult Poker Game](https://www.kdnuggets.com/2020/12/remembering-pluribus-facebook-master-difficult-poker-game.html)

## Overview of Pluribus Data

The goal of this whole module is to give me access to the state inside the `Pluribus` 
[log files](http://kevinwang.us/lets-analyze-pluribuss-hands/).

While I am a test driven developer I am more a data driven developer. There is no better
way to drive logic than with real data. When working on any complex system one of the first
things I look for is state.

## Pluribus Struct

The purpose of the [Pluribus struct](pluribus.rs) is to create a place to store the parsed out data from a Pluribus log
entry. The goal isn't to perform any logic with the data, but simply to store it in a way that makes writing logic code 
easier. 

I originally did this work as a spike. [Wikipedia](https://en.wikipedia.org/wiki/Spike_(software_development)) defines a spike as:

> ...a product development method originating from extreme programming that uses the simplest possible program 
> to explore potential solutions.[1] It is used to determine how much work will be required to solve or work around a software issue.

[Here's the code](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=ff2ed6756899497abe1f4995653b8eff)
for the original spike I wrote exploring this format.

Here's code to dumb out the `Pluribus` structs data from a `Pluribus` log file:

```rust
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;
use pkcore::analysis::store::nubibus::pluribus::Pluribus;

/// This code's origin is from
/// [Rust by Example](https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html).
fn main() {
    // File hosts.txt must exist in the current path
    if let Ok(lines) = read_lines("data/pluribus/raw/sample_game_30.log") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(ip) = line {
                match Pluribus::from_str(ip.as_str()) {
                    Ok(pl) => println!("{}", pl),
                    Err(_) => {}
                }
            }
        }
    }
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

```

This is just an interim data structure. Right now everything is just arrays of state. It is a collection of
one-dimensional data strands. We need to get the state out of `Flatland`.

## Nubibus

I'm actually how surprised I am by how surprised I am in how intricate the data textures are in a single game of poker. 
I've been following the game in one form or another for over 40 years, NLH becoming the defacto game thanks to the 
[World Poker Tour](https://en.wikipedia.org/wiki/World_Poker_Tour) TV show. 

This is one of the biggest dangers for developers... being too familiar with the dynamics of the systems you are 
coding for. Our brains move repetitive sequences into our subconscious mind where it is easier to act upon. Thinking
about things is slow. When they are pure muscle memory, they become much easier. The problem is that you don't notice
the intricacies of the processes. We learn it, then we forget it, except for those rare moments where we have to 
analyze a specific instance of the process.

With that in mind, let's take our flat collection of state and create its mini-universe. What do I mean by that? I am 
talking about the dimensions of the data. What are they.

We have a table where players play. It has seats where the players sit, in the case six. These seats govern the order
with which the players play. It is a merry-go-round, spinning around clockwise, always pointing at one player to act
at a time. Poker is a [turn based strategy game](https://en.wikipedia.org/wiki/Turn-based_strategy).

_This gives me an idea: Poker with no order. Rather than turn based, could you create a version of the game where players
just act, with no specific order? This seems like it would be a fun experiment._ 

### Hand Initialization

Hold it, this is not entirely true. There is an initialization phase that isn't turn based. What happens in that phase?

**Assign the Button**

This is the pointer to the person who gets to act last in every betting round. If this is the first hand at the table, 
there is some method used to determine who gets the button. 

**Blinds**

The `Blinds` are the players to the left of the `Button` that are required to make forced bets at the start of the new 
hand. They are made of the `Small Blind` on the immediate left of the `Button`, and `Big Blind`, sitting on the left
of the `Small Blind`. Generally, the bet of the `Small Blind` is half of the `Big Blind`'s, but this is not always
the case, such as [2/5 No Limit Hold'em](https://www.mattaffleck.com/single-post/2016/09/12/The-Perfect-2-5-No-Limit-Player).

In cash games, the limits don't change. Tournaments, however, have a fixed progression where after a certain period of
time, the blinds go up. The shorter that time is, the more gamble there is to the tournament, since you have less time
to wait for good hands to come your way. 

**Edge Cases**

There is also an edge case where one of the blinds loses all of their money and so leaves the game. We'll deal with this
if/when we need to. 

...

### The Nubibus Struct


### Chips

I would love to create a trait that harnesses the flows of betting, but I am not seeing a clean way to do it given
how I want the `Seat` and `Chips` structs to function. I want the state in Seat to be Cells as opposed to Chips
where it's pure, so if I am changing it, it needs to be mutable. It doesn't feel like it works. 

Right now, I'm going to create it simply so that I have a focus on the contract of betting. 

### Seat

### Walking through play

My goal right now is to bend Nubibus so that I am able to step through every action in a hand. 

### Pointer6Max

The AI generated impl code is good, IF you wanted to create a generic accessor style class, like how
those Java annotation libraries that eliminate the need for getters and setters. BUT, we are going to 
have some real logic to this baby that follows the flow of actual NLH hands. 

```rust
impl Position6MaxPointer {
    /// NOTE AI generated code.
    pub fn current(&self) -> Position6Max {
        self.position.get()
    }

    /// NOTE AI generated code.
    pub fn next(&self) -> Position6Max {
        self.position.get().next()
    }

    /// NOTE AI generated code.
    pub fn increment(&self) {
        self.position.set(self.position.get().next());
    }

    /// NOTE AI generated code.
    pub fn is_over(&self) -> bool {
        self.position.get() == Position6Max::BTN
    }

    /// NOTE AI generated code.
    pub fn is_active(&self, position: Position6Max) -> bool {
        self.active[position as usize - 1].get()
    }

    /// NOTE AI generated code.
    pub fn set_active(&self, position: Position6Max, is_active: bool) {
        self.active[position as usize - 1].set(is_active);
    }

    /// NOTE AI generated code.
    pub fn reset(&self) {
        self.position.set(Position6Max::UTG);
        for _i in 0..6 {
            self.active[_i].set(true);
        }
    }
}
```

#### Rethink

I want all of the pointer logic to have the following structure: Enum > Immutable binary strict > 
Cell based flexible tracker. The underlying state should be able to fit into a simple bitwise datafield.
Problem is that I'm overthinking all this. Better to just get it to work and try for this later on. 

## Thoughts and Initiatives

I'm thinking that I want to start doing my initial unit tests as doctests. The problem is that it requires a lot of
setup, which doctests don't easily facilitate.