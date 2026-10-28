// Alternative: HenryRLee's PokerHandEvaluator (Apache-2.0,
// https://github.com/HenryRLee/PokerHandEvaluator) uses the same 7,462
// equivalence classes as these Cactus Kev tables, with 1 the best hand. It
// reaches them by perfect hash instead of prime products and a binary search.
// Non-flush hands are hashed from a 13-digit rank count (`hash_quinary`,
// cpp/src/hash.c) straight into a `noflush` table. Flushes are found with a
// suit-count hash and indexed by a 13-bit rank mask. It has separate tables for
// 5, 6, 7, 8 and 9 cards and for PLO4/5/6, so there is no best-5-of-N loop. The
// method is written up in Documentation/Algorithm.md. Its README measures the
// 7-card library at about 144 KB of memory. The four tables here come to about
// 60 KB.
/// This code was taken from [Vladislav Supalov's](https://github.com/vsupalov)
/// [pokereval-rs](https://github.com/vsupalov/pokereval-rs) library, which in
/// turn was based on Cactus Kev's (aka [Kevin Suffecool](https://suffe.cool/))
/// [Poker Hand Evaluator](https://suffe.cool/poker/evaluator.html) code in C.
pub mod flushes;
pub mod products;
pub mod unique5;
pub mod values;
