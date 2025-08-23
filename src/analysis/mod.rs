use crate::Card;
use crate::arrays::three::Three;
use crate::play::hole_cards::HoleCards;
use case_evals::CaseEvals;

pub mod case_eval;
pub mod case_evals;
pub mod class;
pub mod eval;
pub mod evals;
pub mod gto;
pub mod hand_rank;
pub mod name;
pub mod omaha;
pub mod outs;
pub mod player_wins;
pub mod store;
pub mod the_nuts;

/// The start of an analysis plugin system.
#[allow(clippy::module_name_repetitions)]
pub trait PlayOut {
    fn play_out_flop(&mut self, hands: &HoleCards, flop: Three);

    /// # Calc PHASE 3;
    ///
    /// I'm really starting to have doubts of the design of this trait. The fact that it
    /// has a very javaee way of handling state... assuming that the implementer has mutable
    /// state that it will just write to seems completely antithetical to rust core principals.
    ///
    /// As an imposter, I am constantly questioning my decisions as I work. I run through them
    /// like a playlist of bad music before I go to bed at night. While this probably isn't healthy,
    /// and will surely be a subject of some future therapy session, it does provide me with the
    /// kind of wonderful inspirations that you just can't have sitting in front of a computer
    /// terminal. _Do people still say computer terminal? Man, I'm really dating myself Na-nu Na-nu!_
    ///
    /// Back when I lived in the Bay Area, I would take the bus to work I had the idea for a way
    /// to deal with a particularly challenging problem for the startup I was working at. I needed
    /// a way to be able to be able to combine content blocks ad hock. We were creating a portal
    /// of comedy content, made up of streaming audio, video, text, and flash games. As I was
    /// staring out at the Golden Gate Bridge from the Bay Bridge, I started picturing a way to
    /// just combine content cells like a spreadsheet, only not just one in two dimensions, but
    /// in infinite ones. It took me a month to create in PHP.
    ///
    /// > SIDEBAR: I used to think only patzers like coded in PHP, and that it was a programming
    /// > language for losers. Than that jerk Zuckerberg showed me what an idiot I was. I also
    /// > thought that I was too good to play in a Latin American orchestra. Then my friend Asaf
    /// > got the principal cellist gig with the State Orchestra of Mexico, where he lives his
    /// > idyllic life doing what he loves, with lots of time to drink beer, eat wonderful food,
    /// > swim, play soccer, and be surrounded by wonderful people. You'd be surprised just how
    /// > stupid your opinions are, especially about yourself.
    ///
    /// Once it was done we could create webpages made out of any content combination with ease. I
    /// was lucky to have Frank with me on the team who took my ideas and ran with it. Being able
    /// to infuse other people's ideas with life is a very important gift, and Frank was the best.
    ///
    /// Later on, when I learned about tagging in blogs, I thought that tags would be a much better
    /// organizational structure than the content IDs we were using. I learned that tags were called
    /// folksonomies, for user driven metadata.
    ///
    /// On a long drive with my friend Steve, I described
    /// my idea for the second version of what was then called `Spew`, later named `FolkEngine`, which
    /// is why that's my twitter handle. He told me that it sounded like Ted Nelson's
    /// [`ZigZag`](https://en.wikipedia.org/wiki/ZigZag_(software)) structure, and he warned me about
    /// [Project Zanadu](https://en.wikipedia.org/wiki/Project_Xanadu), which Wired Magazine wrote
    /// up in [an infamous article](https://www.wired.com/1995/06/xanadu/). He was right.
    ///
    /// The company I wrote the publishing system for, Comedy World, was called "the second worst
    /// dot bomb of all time" by fucked company. Ironically, Project Zanadu was considered a
    /// disaster. I guess I'm in good company, although Ted Nelson is a genius, and one of my heroes,
    /// and I'm just a hack imposter.
    ///
    /// > Comedy World's primary business model was 24/7 streaming comedy radio shows by such greats
    /// > as Sara Bernhard, and Marc Maron. Years later, when I started listening to Air America
    /// > radio I noticed that they were struggling because they wanted to have radio stations
    /// > take their entire lineup of shows. This was the same mistake that Comedy World had made.
    /// > Programmers didn't like being told that our content was all or nothing. In the end
    /// > it doomed us, as it did Air America.
    ///
    /// I haven't written `FolkEngine`. I use the excuse that it was because Nelson patented the
    /// `ZigZag` data structure, and I didn't want to try to avoid it. I thought it was too brilliant.
    /// I still do. The truth is that I didn't feel like I was ready for it. That it was just too
    /// much for my craft at the time. I dream of writing it in Rust, and needed a way to force
    /// myself to really understand the language. And thus, this book.
    ///
    /// ## Refactoring Idea
    ///
    /// What if we combined the work we're doing to calculate the odds with determining the possible
    /// hands at each part of the game? Not going to do this now, but it is work thinking about. It's
    /// getting clear that the code is sluggish and could be optimized. Still, we will hold off for
    /// now. Remember the words of the great [Donald Knuth](https://wiki.c2.com/?PrematureOptimization):
    ///
    /// > Programmers waste enormous amounts of time thinking about, or worrying about, the speed of noncritical parts of their programs, and these attempts at efficiency actually have a strong negative impact when debugging and maintenance are considered. We should forget about small efficiencies, say about 97% of the time: premature optimization is the root of all evil. Yet we should not pass up our opportunities in that critical 3%.
    ///
    ///
    fn play_out_turn(&mut self, hands: &HoleCards, flop: Three, turn: Card);

    fn case_evals_flop(&self, hands: &HoleCards, flop: Three) -> CaseEvals;

    fn case_evals_turn(&self, hands: &HoleCards, flop: Three, turn: Card) -> CaseEvals;
}
