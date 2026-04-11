# EPIC-19A: Mutants Sidequest

This week, I stumbled across Elliott Clark's beautiful [rs-poker](https://github.com/elliottneilclark/rs-poker) library.
It came about as I was looking for formats to serialize poker hands. Clearly Pluribus' format, which we already support,
is way too concise to be practical: 

`STATE:1:ffffr300f:8sQc|2s8d|7dTs|5d8h|2h9s|6cQd:100|-100|0|0|0|0:Gogo|Budd|Eddie|Bill|Pluribus|MrWhite`

What came up from searches were the following:

- [Poker Hand History (PHH)](https://phh.readthedocs.io/en/stable/)
- [Open Hand History (OHH)](https://hh-specs.handhistory.org/)

One implementation of OHH in Rust was the [rs-poker](https://github.com/elliottneilclark/rs-poker) library.
It's always a hit to the ego when you see someone doing something better than you. But then you step back and consider
the situation organically. rs-poker has a very clear, definable goal, executed by someone who's got insane foo and 
street cred. I mean, come on... the guys a [Facebook vet](https://elliottclark.info/) who commits in
 [Rust](https://github.com/elliottneilclark/rs-poker) and runs a company that uses
[Elixer](https://github.com/batteries-included). My two favorite languages, and I've never made a dime coding in them.
Respect. 

There seems to always be a 10 to 1 ratio of people that do the
[Simpson's did it](https://www.youtube.com/watch?v=PHr-C-qIfRU) routine on it vs actually talking about the idea.
Take a step back, examine what you're trying to do, and focus on the work's goals. What is your imperative?

> Talking about the imperative is one of my crazy idea in system evaluation and design. Don't get me started talking
> about Beethoven's 5th.

For [pkcore](https://github.com/ImperialBower/pkcore) it was simple, which makes it very complex. I want a universal
core poker library that I can use as a foundation for many different use cases: Table mechanics, hand evaluation, 
poker solvers, [GTO tools](https://github.com/ImperialBower/pkgto-web), networked AI and other bot game play, and cheat
detection. It should be able to run on servers, web pages, 
on mobile devices etc. I want to use it to be able to play poker with my friends using nothing more than text messages. 
I want to see an entire simulated casino running based on it so I can demonstrate ideas like Observability and testing as
well as a playground for learning about the [game theory](https://github.com/ImperialBower/pkkuhn-web) and ML.
I want something that I can play with, knowing that I can break things at will because the core is solid AF. It's a
test of an architectural idea that I've believed in my entire career: domains should have a battle tested, rock solid
kernel definition of truth. The problem is, it's rare when you have a blank canvas to paint on. For me it was 
an early start-up where I created what I still feel is one of my best ideas. 

[rs-poker](https://github.com/elliottneilclark/rs-poker) is solving a very specific problem, and it is a demonstration
of all my flaws as a developer. It executes on an idea brilliantly. I'm a dreamer with ideas floating around in my 
head for decades. I've spent over five years on this shit coding in a language I knew very little about just test-driving
my way through shit blindly as a trauma response to the pandemic. Rust was my safe place. My way to take control of
something in a world spiraling out of control. 

So what did I do when I found [rs-poker](https://github.com/elliottneilclark/rs-poker)? I didn't look through the source
code. I didn't read the docs. I looked at how are they testing this shit. And I found some cool shit that I'd never 
heard about before.

> Your code is the hero. Your tests tell the hero's journey.

- My homage to Joseph Campbell

[rs-poker](https://github.com/elliottneilclark/rs-poker) used some really cool shit, most of which I'd never heard of
before. 

It uses [mise-en-place](https://mise.jdx.dev/) to manage [builds](https://github.com/elliottneilclark/rs-poker/blob/master/mise.toml).
My godson turned me on to it as a replacement for [asdf](https://asdf-vm.com/), managing the versions of the programming
languages that I use. As a "polyglot" programmer, I am constantly dealing with multiple languages, often requiring
very specific versions. asdf was my goto, and mise has been an excellent drop in replacement. But managing builds??!!

A year ago I was employing my own, roll your own pattern, which is always sus. Basically, I added `./bin` to my path,
and dropped in executable shell scripts that I could just type on command. There is also a folder on the path for scripts
that I found myself running all the time. Looking back on it, it's clear how much of an antipattern in was. I haven't
had need to use the [Ethereum test net](https://github.com/folkengine/bingbang/blob/main/drink) to test blockchain
contracts in many years, and the one it uses has been long shut down. Still, the efficacy of just typing `drink` and have
your coffers filled was nice. 

Then, for Rust projects I stumbled across [cargo-make](https://github.com/sagiegurari/cargo-make). I got rid of my
`bin` folders, and tried it out. It worked great. But then a former coworker that I respect a lot shared a [Rust library
he was working](https://github.com/rubberduck203/duklog) on that had one very interesting tool choice:
[GNU Make](https://www.gnu.org/software/make/). One of the reasons that I love go as a programming language is that
make is a standard part of its build process. 

In the end, I decided to use Make. I like the idea of using mise to handle the languages I use, but I don't want it to
infuse everything I do. It's too niche, IMHO. But that's just me. Sure, Make's get off my lawn old, but it's universal, and TBH, do I really need another
rewrite it in Rust version of an old standard? All of them work. I like, however, that I am setting on universal tools
across the languages I code in, when they make sense. 