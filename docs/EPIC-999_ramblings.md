# Ramblings

## My dev hardware

This is why I have a 13" M1 and my System76 linux tank. The M1 is going to be a major defect vector for a long time. I am still stepping on landmines after a year.
BUT, as I sit here outside a cafe in Burbank, I am grateful AF for how portable this M1 is.
One of my power moves back in my startup days was to spell out my hardware demands in my contract before I would sign. Two machines, top of the line Mac and a boss-ass Linux box. Your odds of getting that before you sign are less than 1%. Before, closer to 90%.
My person rule is that I have a box to code on that is the OS I am deploying to. I had mad respect for Verizon that they gave you Solaris workstations if you were coding for their servers.
My dream is that someday I will return to my roots and only work on a FreeBSD machine. I hate that all these Apple and Docker posers make so much bank off of shit that they pioneered decades ago.
:nice:

Sorry for rambling, but this ushered in a thought:
The reason that they don’t like to have you work on machines that match what you are deploying to is cost, but the resulting defects that this allows are some of the most brutal there are, and end up costing them hundreds of thousands of dollars which could have been saved if they spent a few thousand on some FUCKING RAM for their developers and dev machines that aren’t some locked down Windows shit machines or bottom of the line macbooks. (edited)

This tree keeps dropping little flowers on my laptop’s keyboard as I type. This is one of those perfect moments that Spalding Gray talked about.

![Perfect Moment](files/perfect_moment.jpg)

## Leet Dev Image Hacks

Here's a dev hack that you can use if you're trying to emote a certain amount of
h4ck3r l33tness to your game: develop using hard core old school tools such as
VI or Emacs... or even extra double hardcode,
[Emacs evil mode](https://www.emacswiki.org/emacs/Evil), which is Emacs with
VI commands. Being able to develop without the training wheels tha a hard core
IDE like IntelliJ has to offer will set you apart from the vast majority of
developers out there, including me. PLUS, being able to wage endless pointless
verbal wars over the superiority of VI over Emacs is an essential skill within
just about any development shop. Personally, anyone who starts coding strainght
from either of these environments earns my mad respect. They've clearly done
the work.

/// ## The Gold Standard
///
/// There is a state in any codebase that I like to call the Gold Standard. It's very hard to achieve but when you do give
/// yourself and the people you work with. Here it is:
///
/// > **_Any time you change how your code interacts with any other parts of the code it should make a test that passed in
/// > the past fail._**
///
/// Now I am not saying that you have to always meet this standard, just to watch for it. If you suddenly do a significant
/// change to the functionality of your code and no test fails, try to figure out why.