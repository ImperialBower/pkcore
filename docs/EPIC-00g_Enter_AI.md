# Enter AI

As everyone at this point knows, some really freaky shit has started happening in the programming
space, and to be real the entire reality space over the last year. AI programming tools have become 
supercharged.

I subscribed to [GitHub Copilot](https://github.com/features/copilot) over a year ago. I found it
an interesting, if often misguided tool, that would extend autocomplete. Luckily, my extreme linting
levels and insance amount of test coverage would insulate me from the worse of its habits. It was
extremely useful for one of my more crazy ideas, of creating a macro that parsed 
[GTO card combos](https://github.com/ImperialBower/pkcore/blob/49ae34d2534fded0c3eade536b7b2238cb8be71e/src/macros.rs#L55).
The work itself was extremely tedious, and the autocomplate for something tedious like this was 
pure joy. I didn't need it to think for me. What I wanted was something to make my silly hobby project
easier to create.

```txt
(22+) => {
    Twos::from($crate::analysis::gto::DEUCES.to_vec()).extend(
        &Twos::from($crate::analysis::gto::TREYS.to_vec()).extend(
            &Twos::from($crate::analysis::gto::FOURS.to_vec()).extend(
                &Twos::from($crate::analysis::gto::FIVES.to_vec()).extend(
                    &Twos::from($crate::analysis::gto::SIXES.to_vec()).extend(
                        &Twos::from($crate::analysis::gto::SEVENS.to_vec()).extend(
                            &Twos::from($crate::analysis::gto::EIGHTS.to_vec()).extend(
                                &Twos::from($crate::analysis::gto::NINES.to_vec()).extend(
                                    &Twos::from($crate::analysis::gto::TENS.to_vec()).extend(
                                        &Twos::from($crate::analysis::gto::JJ.to_vec()).extend(
                                            &Twos::from($crate::analysis::gto::QQ.to_vec()).extend(
                                                &Twos::from($crate::analysis::gto::KK.to_vec()).extend(
                                                    &Twos::from($crate::analysis::gto::AA.to_vec())
                                                )
                                            )
                                        )
                                    )
                                )
                            )
                        )
                    )
                )
            )
        )
    )
};
```

I mean, yeah, let the bot handle the stupid stuff.

The Rust programming language itself was the best cop on the block, enforcing the reality I wanted
and preventing the AI to rando its way through the problems I was working through. When I use terms 
like Card::TREY and Card::DEUCE, CoPilot would always call them THREE and TWO, and the compilor
would error out so I could fix the problem. Anytime myself, or the AI did something stupid, Rust
would scream, and often suggest the perfect replacement.

This is in stark contrast to the AI work I've seen in the weakly typed Python language. Python's
flexibility has turbo charged development work for the data inclined. It's also enabled the creation
of some of the worse code in the world. As a slop enabler, it's a 10 out of 10. 

Even the stupid stuff was no match for Rust's Clippy linter. Hacky `for loops` would get called out 
without mercy, giving me the benefit of working through problems fast, while keeping things tight
when ready to commit. As a long time addict or O'Reilly Nutshell books and StackOverflow, the fact
that I could get sub-optimal suggestions right from my editor was cool AF.

## [Claude Code Take the Wheel](https://suno.com/song/5894a10b-272b-44d0-8ca9-1f9a049ec533)

Enter [Anthropic's Claude](https://claude.com/product/claude-code). 

I started using it for a project at work. 