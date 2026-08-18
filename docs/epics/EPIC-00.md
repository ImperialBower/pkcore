# EPIC 00

## Outline

* Got rust?
  * Cargo, your new best friend
  * Cargo clippy BEAST MODE
  * Cargo fmt
    * STORY TIME: Why I love clean code. (Migraines)
* [Setup wasm](https://rustwasm.github.io/docs/book/game-of-life/setup.html).
* Why Rust?
  * Inverting the curve
  * THE BIG IDEA: Better to eliminate a problem than to solve it.
  * Rust TDD loop
    * define
    * create fn sig returning default value
    * create failing test valid on expected value
    * Make test green
    * any more boundary conditions?
    * refactor
    * draw negative boundary refactor to Result for overdraw
* Letting the IDE do a lot of the work (Mad Dog Murdock)
  * Compare CLion to VSCode

## Reading material

## Notes on the Imposter

The truth is that no one is as hard on us as ourselves, and the smarter you are,
the more brutal you can be with yourself. This is why so many idiots are in
management.

In the grand arena of life we are always really competing with ourselves, and
no matter how good we do, we could always do better. This is why unless you
learn from your mistakes, you will never achieve mastery of anything.

This is not entirely true. There are people who are gifted with a natural
mastery. People like the Chess world champion
[José Raúl Capablanca](https://en.wikipedia.org/wiki/Jos%C3%A9_Ra%C3%BAl_Capablanca),
who beat his father the first time he played chess. He was four years old. I
am not one of those people, and to be honest, the skills of the prodigy do
little good in the area of software, because the whole point of the things we
build is for them to be included in other, more complex systems. The human brain
is a wonderfully brilliant machine. The computer can count to one really really
fast. It's just not that bright.

Thus, we must learn from our mistakes if we are going to improve.

## Themes

### Creating a Safe Place

CI tools, tests, give yourself a safe place to fail and learn.

### Beware those with The Answer™

You will meet many people in your journey through life that will offer you
The Answer™. They will tell you the right way to do things. That however you
are doing things, you are doing them wrong. These people are lauded...
celebrated... held up as geniuses... innovators in their
fields, and many people will be drawn to them because they're looking for
The Answer™, and are frustrated by how complicated everything is. I understand
the attraction. Life would be so much easier if I had The Answer™;
if I could just do this one thing and all of my problems would vanish.

Be careful. The Answer™ means that you stop doubting. The moment you decide
that you know everything... that's the true moment you have truly failed.

## Ways to read

I've written this book so that you can approach it in a number of ways. It seems
like most programming books these days are designed to be interactive, because
that's a great way to learn stuff. The problem with this is that you need to
be sitting in front of a computer while you read the book.

However, I want to create something that can be enjoyed in a number of ways.

* Interactively, so that you can build up some Rust muscle memory.
* Casually, so that you can get the experience of something just watching me code; sort of like a book version of a Twitch stream.

### Perspective

* The Booth - God Mode
* Player
* Spectator - sees only what everybody sees.

### Your Tools

#### The Command Line

    In the Beginning...was the Command Line 
    -- Neal Stephenson

The command line is your first, and most important tool as a programmer. It has
been my constant companion over the course of my career. There are many command
lines. This one is mine. _examples_ ...

##### rustup

## The Crazy

I'm a programmer because I love programming. You need to find your crazy. It's
not for everyone, but that's OK. You can make those big tech bucks (LOL) without
touching code at all. Nerd wrangling is an art all its own.

## CI

Set yourself so that you can be stupid. Don't require yourself to have to
remember to constantly run `cargo test && cargo clippy && cargo fmt.` Assume
that failure is the normal human state, and you will give yourself a little
more room to breathe. Yes, I constantly run test and clippy and fmt, but if
I don't, I have my trusty GitHub actions robot (_I call mine R4-P20_) to let me
know if I forgot something, making sure that I won't push out something that
will truly screw things up. Truly great programmers never make mistakes and are
always infallible. They work for places that pay millions of dollars and never
ever ever do things like enable coups or create automated surveillance states.
You will meet these people all the time. Alas, we are merely imposters, and so we
rely on our tools and craft to have our backs. Enjoy the freedom of not being
perfect in every way. Being Superman means that you have to deal with the pain
in the ass of being constantly asked to save the world. As imposters, we can
go out and do things like walk our dogs, take naps and hug the ones we love.

_Memento mori._

### Time Pressure

One of the things that you are going to hit upon over the course of your
adventures in software development is time pressure. The best insurance policy
you can take out against being crushed under the weight of this pressure are
the techniques that give you fast, independent feedback. The systems we are
building today are massively complex, and honestly beyond the understanding
of the people building the system or selling it. Our brains simply aren't
designed to hold it, and without a direct connection to our senses, we are
blind, deaf, and dumb as to what's going on inside it. This is where your craft
is so essential. A solid foundation of tests. Tools that help you keep your
code clean. CI servers that alert you the moment somebody tries to commit
something that will break your system. Logging systems that allow you to
explore the state of your system in aggregate.

People will tell you to worry about these things later. Worry about them now.
Start with a strong foundation. Take the time while you have it. Don't be the
person kicking themselves later for not doing the things that you know you
should have.

## Perspectives

In any sort of analysis of a state in flux, it is important to understand the
different perspectives that the system offers to view, interact with, and
analize. We are starting from the landscape of Texas Hold'Em Poker. We are not,
as of yet, worrying about bets or stack size; simply focusing on the odds of
winning in pure, card based terms. Later on, we will want to add that to our
system's domain map, but before that, we need to lock down our ability to
analyze a hand in pure terms.

Now, what does that mean? What are the perspectives that a Texas Hold'Em can
have in pure, probability terms?

### Booth View

One of the big revolutions that came about in terms of how the game of poker was
perceived by the public was brought about by the genius of
[Henry Orenstein](https://www.usbets.com/remembering-poker-pioneer-henry-orenstein/).
He came up with the bright idea of making it possible for spectators of poker
games to see what they players hands were, using cameras built into the tables.

This is the first perspective that we are going to code towards.

#### Aside

Another way of thinking of perspectives is what is called in user-centered
design and marketing as a
[Persona](<https://en.wikipedia.org/wiki/Persona_(user_experience)>. I find these
sorts of organizational tools in organized, corporate settings as useful; but
this is not a book based on that context. I am not writing to make you be a
better team player, able to create synergy between various knowledge silos in a
way to maximize customer value. I am writing about how I code, and how I think.

My brain naturally revolts internally against efforts of collective group think,
designed to make us work together in lock step. That's my day job. I like
languages, and keeping my brain flexible. This is not user centered design. The
only user of this system right now is me, and hopefully, later on the users will
include you, as you play around with the code I am writing.

We are not building a product. We are not moulding our system to maximize profit.
We are building a digital representation of an intellectual domain. What we are
creating is a layer below the work of marketing, and product owners, and venture
capitalists, who filter everything they do through the dust colored glasses of
money.

Yes, yes... I know this is a harsh collection of words. I really appreciate
the people for the companies I work with who do these jobs. They are essential
when working in our neoliberal dystopia. We get paid to sell things, and
every dimension of our work needs to be focused on that principal. However,
IMNSHO, there is a gravity to that dimension, which, when introduced too early,
warps a domain in ways that restrict its flexibility for later innovation.

You will hear things like:

* I'm not paying testing. I'm paying for a working system.
* We don't need to worry about performance now. We can focus on that after we have a functioning product.
* Why are you spending so much time gold plating this product? We need this out the door NOW!

These opinions have more and more merit, the farther you get away from the core
center of a system's domain. However, as I've said before, I am not a test
driven developer. I am a domain driven developer that is test obsessed. I focus
first on creating a set of code that as best as I can, reflects the texture of a
domain. Once I feel like I have that, THEN I start worrying about leveraging it
for customer facing applications.

Google was build on the foundations of a paper written at Stanford. Facebook
was build on the backs of a bunch of horny Harvard students trying to create
better ways to creep on unsuspecting women.

Your domain is the seed. Your product is the flower. Focus on the seed, and you
will have a garden full of flowers. _Note, these opinions are heretical in my
chosen profession. We are driven by entitled type-A sociopathic con artists
enabled by our particularly tulipian form of capitalism. Don't hate the player,
hate the game. Know that truly great things aren't built from greed, but from
passion, and love, and a desire to bring a little beauty to the world._

Now, granted, this domain we are splashing around with is professional gambling,
one of the most toxic environments in the world. We acknowledge that. Maybe I am
a bad person for it, but I love playing poker. I want to build tools that will
help me be a better poker player. Maybe I will make some money from this... maybe
not. In any event, I will have had fun doing it, and let's just see where this
leads us.

The thing that really worries me, is that I may just enjoy coding about poker
more than I do playing it. Yes, our of the true joys in my life has been those
moments when a player throws their hand at me in disgust because they had a bad
read on my playing style. Those memories are sweet, but fleeting. Writing has
the potential to last longer, AND, if done well, provide more of those wonderful,
fleeting moments.

### Player View

Our second targetted perspective is from that of the player. This is where
we know only a single hand. How to we build a system that can offer insights
from this view? What are the key differences from the Booth View?

First off, we need to acknowledge that this view isn't really possible until
we feel we've locked down a good first pass at the Booth View. It will give us
the foundation to start to add more possibilities to the game play.

Modern players call this Game Theory Optimal, or GTO for short. This is where
you don't just focus on trying to guess what single hand your opponents have,
but a range of possible hands, and what they dictate in terms of optimal play
from a probability perspetive.

Think of it as opening up the Multiverse of Madness from Doctor Strange. You
aren't playing one hand against one player. You are playing against every
possible hand that player has, trying to figure out from a probability
perspective, what is the most rational decision to play.

The way you avoid being swallowed by the madness is to have mental muscles that
help you remove possibilities. We want our software to be a study aid that
will help you be better and flexing those muscles. For this we will introduce
ranges of hands, and add methods of quickly eliminating possibilities, and
distilling things down.

This is where we introduce our first
[domain-specific language](https://en.wikipedia.org/wiki/Domain-specific_language).
We want to be able to type in things like `AJo+` and have our tool offer up
information about the possibilities that those dimensions offer for our
opponents. Now `AJo+` will translate to a non-pocket pair if Ace and Jack of
any suit or better where the two cards are not of the same suit; aka `AJ AQ AK`.
`A9s+` would be Ace and Nine or better where the two cards are of the same suit,
and `AT+` means Ace and Ten or better where the two cards can be of any suit, the
same or not.

### Spectator View

This is where you are in the crowd, and don't know what any of the players are
holding.

While Henry Orenstein's innovations where amazing for the game, we did lose a
fascinating perspective of the game. If you have a PokerGo subscription, you
can watch a young Phil Hellmuth joining the TV booth at the WSOP to talk about
the play of the great Stu Ungar at the final table where he didn't know what
the players were holding at all.

This perspective requires that we zoom our system landscape's lens out to
have data about stack sizes, and blinds, and bet sizes. Professional poker
players know a tremendous amount about what is going on in a card game
without seeing the cards at all. Is it possible for us to create tools that will
help us get better at those skills? It is very possible, that this perspective
is beyond our grasp, but I at least what to acknowldge that it is possible.

## Warning

I am the lover of strange writings. I was raised by a single, lesbian opera
singer who's favorite author is Gertrude Stein. Zippy the Pinhead, Quasi at the
Quackadero, Weirdo, Neat Stuff, American Flagg, Charles Bukowski, Armistead
Maupin, Hunter S. Thompson, Gracie Allen, The Residents, John Waters,
Malcolm X and Lenny Bruce are the perspectives that speak to me most. I have
always been an outsider, and treasure that perspective.

Also, when you pair with me, I will curse. I will say shit, and fuck, and fart,
and if you are offended by these things, let me know. Personally, given we live
in a world where one of the political parties is openly condemning my mom's
lifestyle, and kids being gunned down in their classrooms is the norm, I could
give a flying fuck about your delicate sensibilities. I have zero tolerance
for racism, sexism, misogyny, and prejudice toward people just trying to live the
most authentic version of their lives. If you have a problem with that, talk to
someone about how uncomfortable you get when you see two people of the same sex
holding hands, or why it offends you to have people of different races being
happy and living their lives. You may discover something very interesting about
yourself. *INSERT PROMO CODE FOR BETTER HEALTH HERE*

Consider yourself warned ;-)

## Test Driving, Thin Slicing, and My Chunnel Approach

Domain Driven Design, the sweet spot between TDD and Architecture.

Perspective: [Why user stories and the thin slice are a lie…](https://hurlbert.medium.com/why-user-stories-and-the-thin-slice-are-a-lie-a3c393bde81#:~:text=In%20Agile%20there%20is%20lore,and%20implement%20the%20user%20story.)

The structure

* Core domain definitions
  * Hardened core API
    * Architecture defining system communication and interfaces
      * Display... _i._e. applications
