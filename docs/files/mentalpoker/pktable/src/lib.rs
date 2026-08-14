//! # pktable — protocol + game driver shared by the relay and the client.
//!
//! Wire format (one event per line, '|'-separated):
//!
//!   client -> relay   P|author|kind|payload
//!   relay  -> client  W|seat                          (private welcome)
//!   relay  -> all     E|seq|prev_hex|author|kind|payload
//!
//! The relay assigns `seq` and `prev` (the rolling FNV-64 head before this
//! event) and appends the line to the chain. Every client independently
//! recomputes the chain and rejects any event whose `prev` doesn't match its
//! own head — the toy version of the signed hash-chained log. Signatures are
//! mocked out (author byte only); ed25519 slots in where noted.
//!
//! Event kinds: J join · K keyshare · S shuffle(deck) · R reveal(token) ·
//! A action (C check, B:n bet, L call, F fold).

use pkcore_mp::card::{Card, DECK_ARRAY};
use pkcore_mp::{CardCrypto, PlainMasked, PlainToken, PlaintextCrypto, Seat};
use std::collections::{BTreeSet, HashMap, HashSet};

pub const SEATS: usize = 2; // heads-up skeleton
pub const HOLE_SLOTS: [[usize; 2]; 2] = [[0, 2], [1, 3]]; // alternate deal
pub const FLOP: [usize; 3] = [5, 6, 7]; // slot 4 burned
pub const TURN: usize = 8;
pub const RIVER: usize = 9;

// ── hash chain ───────────────────────────────────────────────────────────────

pub const CHAIN_INIT: u64 = 0xcbf29ce484222325;

/// Fold a line into the chain head (FNV-1a). Real impl: SHA-256 over the
/// serialized signed envelope.
pub fn chain_fold(head: u64, line: &str) -> u64 {
    let mut h = head;
    for b in line.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// ── wire event ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct WireEvent {
    pub seq: u64,
    pub prev: u64,
    pub author: Seat,
    pub kind: char,
    pub payload: String,
}

impl WireEvent {
    pub fn to_line(&self) -> String {
        format!("E|{}|{:016x}|{}|{}|{}", self.seq, self.prev, self.author, self.kind, self.payload)
    }
    pub fn parse(line: &str) -> Option<WireEvent> {
        let mut it = line.splitn(6, '|');
        if it.next()? != "E" {
            return None;
        }
        Some(WireEvent {
            seq: it.next()?.parse().ok()?,
            prev: u64::from_str_radix(it.next()?, 16).ok()?,
            author: it.next()?.parse().ok()?,
            kind: it.next()?.chars().next()?,
            payload: it.next().unwrap_or("").to_string(),
        })
    }
}

// ── payload (de)serialization for the mock crypto types ─────────────────────

pub fn masked_to_str(c: &PlainMasked) -> String {
    let locks: Vec<String> = c.padlocks.iter().map(|s| s.to_string()).collect();
    format!("{}:{}", c.card_ix, locks.join(";"))
}

pub fn masked_from_str(s: &str) -> Option<PlainMasked> {
    let (ix, locks) = s.split_once(':')?;
    let padlocks: BTreeSet<Seat> = if locks.is_empty() {
        BTreeSet::new()
    } else {
        locks.split(';').filter_map(|x| x.parse().ok()).collect()
    };
    Some(PlainMasked { card_ix: ix.parse().ok()?, padlocks })
}

pub fn deck_to_str(deck: &[PlainMasked]) -> String {
    deck.iter().map(masked_to_str).collect::<Vec<_>>().join(",")
}

pub fn deck_from_str(s: &str) -> Option<Vec<PlainMasked>> {
    s.split(',').map(masked_from_str).collect()
}

// ── the replicated game state machine ────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Street {
    Setup,
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAct {
    Check,
    Bet(u32),
    Call,
    Fold,
}

impl PlayerAct {
    pub fn to_payload(self) -> String {
        match self {
            PlayerAct::Check => "C".into(),
            PlayerAct::Bet(n) => format!("B:{n}"),
            PlayerAct::Call => "L".into(),
            PlayerAct::Fold => "F".into(),
        }
    }
    pub fn parse(s: &str) -> Option<PlayerAct> {
        match s.chars().next()? {
            'C' => Some(PlayerAct::Check),
            'L' => Some(PlayerAct::Call),
            'F' => Some(PlayerAct::Fold),
            'B' => Some(PlayerAct::Bet(s.get(2..)?.parse().ok()?)),
            _ => None,
        }
    }
}

/// Everything a node derives from replaying the log — identical on all peers —
/// plus this node's private knowledge (its decoded hole cards).
pub struct GameState {
    pub me: Seat,
    pub crypto: PlaintextCrypto,
    pub head: u64,
    pub joined: BTreeSet<Seat>,
    pub keyshares: BTreeSet<Seat>,
    pub agg: BTreeSet<Seat>,
    pub shuffles: usize,
    pub deck: Vec<PlainMasked>,
    pub tokens: HashMap<usize, Vec<PlainToken>>, // slot -> received tokens
    pub public_cards: HashMap<usize, Card>,      // fully-opened slots
    pub my_hole: Vec<Card>,                      // private: my decoded holes
    pub street: Street,
    pub acts_this_street: Vec<(Seat, PlayerAct)>,
    pub pot: u32,
    pub stacks: [u32; SEATS],
    pub pending_bet: Option<u32>,
    pub winner_by_fold: Option<Seat>,
    pub shown: HashMap<Seat, Vec<Card>>, // showdown hands
    published: HashSet<String>,          // duty keys already emitted by *me*
}

/// Something this node owes the protocol right now.
#[derive(Clone, Debug)]
pub enum Duty {
    Publish { kind: char, payload: String },
    /// It's my turn to bet — surface a prompt (or let the bot decide).
    AwaitAction,
}

impl GameState {
    pub fn new(me: Seat) -> Self {
        Self {
            me,
            crypto: PlaintextCrypto::new(),
            head: CHAIN_INIT,
            joined: BTreeSet::new(),
            keyshares: BTreeSet::new(),
            agg: BTreeSet::new(),
            shuffles: 0,
            deck: Vec::new(),
            tokens: HashMap::new(),
            public_cards: HashMap::new(),
            my_hole: Vec::new(),
            street: Street::Setup,
            acts_this_street: Vec::new(),
            pot: 0,
            stacks: [1000; SEATS],
            pending_bet: None,
            winner_by_fold: None,
            shown: HashMap::new(),
            published: HashSet::new(),
        }
    }

    /// Verify the chain link and apply one event. Every peer runs this
    /// identically — the replicated state machine's `fold`.
    pub fn apply(&mut self, ev: &WireEvent) -> Result<Vec<String>, String> {
        if ev.prev != self.head {
            return Err(format!(
                "chain break at seq {}: event prev {:016x} != my head {:016x}",
                ev.seq, ev.prev, self.head
            ));
        }
        self.head = chain_fold(self.head, &ev.to_line());
        let mut notes = Vec::new();

        match ev.kind {
            'J' => {
                self.joined.insert(ev.author);
                notes.push(format!("seat {} joined ({} at table)", ev.author, self.joined.len()));
            }
            'K' => {
                let pk: Seat = ev.payload.parse().map_err(|_| "bad keyshare")?;
                // Real impl: verify the Schnorr proof of knowledge here.
                self.crypto.verify_key(&pk, &()).map_err(|e| e.to_string())?;
                self.keyshares.insert(pk);
                if self.keyshares.len() == SEATS {
                    let pks: Vec<Seat> = self.keyshares.iter().copied().collect();
                    self.agg = self.crypto.aggregate(&pks);
                    notes.push("all key shares in; aggregate key formed".into());
                }
            }
            'S' => {
                let next = deck_from_str(&ev.payload).ok_or("bad deck")?;
                if !self.deck.is_empty() {
                    // Real impl: verify the Bayer–Groth shuffle argument here.
                    self.crypto
                        .verify_shuffle(&self.agg, &self.deck, &next, &())
                        .map_err(|e| format!("shuffle rejected: {e}"))?;
                }
                self.deck = next;
                self.shuffles += 1;
                notes.push(format!("shuffle {}/{} verified", self.shuffles, SEATS));
                if self.shuffles == SEATS {
                    self.street = Street::Preflop;
                    notes.push("deck fixed; dealing".into());
                }
            }
            'R' => {
                let mut it = ev.payload.split('|');
                let slot: usize = it.next().and_then(|x| x.parse().ok()).ok_or("bad slot")?;
                let _target = it.next().ok_or("bad target")?;
                let tok_seat: Seat =
                    it.next().and_then(|x| x.parse().ok()).ok_or("bad token seat")?;
                let tok_ix: u8 = it.next().and_then(|x| x.parse().ok()).ok_or("bad token ix")?;
                let token = PlainToken { seat: tok_seat, card_ix: tok_ix };
                // Real impl: verify the Chaum–Pedersen proof against author's pk.
                self.crypto
                    .verify_reveal_token(&ev.author, &self.deck[slot], &token)
                    .map_err(|e| format!("token rejected: {e}"))?;
                self.tokens.entry(slot).or_default().push(token);
                self.after_token(slot, &mut notes)?;
            }
            'A' => {
                let act = PlayerAct::parse(&ev.payload).ok_or("bad action")?;
                self.apply_action(ev.author, act, &mut notes)?;
            }
            // 'N' = pass: a no-op that hands the publishing baton onward.
            // Relay-based tables never need it (the relay orders); serverless
            // transports like the QR table use strict alternation, and a seat
            // with nothing to publish passes.
            'N' => {}
            k => return Err(format!("unknown event kind {k}")),
        }
        Ok(notes)
    }

    fn after_token(&mut self, slot: usize, notes: &mut Vec<String>) -> Result<(), String> {
        let have = self.tokens.get(&slot).map(|v| v.len()).unwrap_or(0);
        // My hole slot: others' n−1 tokens + my own (never published) opens it.
        if let Some(_) = HOLE_SLOTS[self.me as usize].iter().position(|&s| s == slot) {
            if have == SEATS - 1 {
                let mut toks = self.tokens[&slot].clone();
                toks.push(self.crypto.reveal_token(&self.me, &self.me, &self.deck[slot]));
                let open = self.crypto.unmask(&self.deck[slot], &toks).map_err(|e| e.to_string())?;
                let card = self.crypto.decode(&open).map_err(|e| e.to_string())?;
                self.my_hole.push(card);
                notes.push(format!("hole card revealed to me: {card}"));
            }
            return Ok(());
        }
        // Public slots (board or showdown): all n tokens open it for everyone.
        if have == SEATS {
            let open = self
                .crypto
                .unmask(&self.deck[slot], &self.tokens[&slot])
                .map_err(|e| e.to_string())?;
            let card = self.crypto.decode(&open).map_err(|e| e.to_string())?;
            self.public_cards.insert(slot, card);
            notes.push(format!("slot {slot} public: {card}"));
            self.maybe_advance_after_reveal(notes);
        }
        Ok(())
    }

    fn maybe_advance_after_reveal(&mut self, notes: &mut Vec<String>) {
        let done = |slots: &[usize], pc: &HashMap<usize, Card>| slots.iter().all(|s| pc.contains_key(s));
        match self.street {
            Street::Flop if done(&FLOP, &self.public_cards) => {
                notes.push(format!("flop: {}", self.board_str()));
            }
            Street::Turn if self.public_cards.contains_key(&TURN) => {
                notes.push(format!("turn: {}", self.board_str()));
            }
            Street::River if self.public_cards.contains_key(&RIVER) => {
                notes.push(format!("river: {}", self.board_str()));
            }
            Street::Showdown => {
                // A showdown reveal: attribute opened hole cards to their seat.
                for (seat, slots) in HOLE_SLOTS.iter().enumerate() {
                    if seat as Seat == self.me {
                        continue;
                    }
                    let cards: Vec<Card> =
                        slots.iter().filter_map(|s| self.public_cards.get(s)).copied().collect();
                    if cards.len() == 2 {
                        self.shown.insert(seat as Seat, cards);
                    }
                }
                self.shown.insert(self.me, self.my_hole.clone());
                if self.shown.len() == SEATS {
                    self.street = Street::Over;
                    notes.push("showdown complete".into());
                }
            }
            _ => {}
        }
    }

    fn apply_action(
        &mut self,
        seat: Seat,
        act: PlayerAct,
        notes: &mut Vec<String>,
    ) -> Result<(), String> {
        if self.to_act() != Some(seat) {
            return Err(format!("seat {seat} acted out of turn"));
        }
        match act {
            PlayerAct::Bet(n) => {
                self.stacks[seat as usize] -= n;
                self.pot += n;
                self.pending_bet = Some(n);
            }
            PlayerAct::Call => {
                let n = self.pending_bet.take().ok_or("call with no bet")?;
                self.stacks[seat as usize] -= n;
                self.pot += n;
            }
            PlayerAct::Fold => {
                self.winner_by_fold = Some(1 - seat);
                self.street = Street::Over;
            }
            PlayerAct::Check => {
                if self.pending_bet.is_some() {
                    return Err("check facing a bet".into());
                }
            }
        }
        notes.push(format!("seat {seat}: {act:?}  (pot {})", self.pot));
        self.acts_this_street.push((seat, act));
        if self.street != Street::Over && self.acts_this_street.len() == SEATS {
            self.advance_street(notes);
        }
        Ok(())
    }

    fn advance_street(&mut self, notes: &mut Vec<String>) {
        self.acts_this_street.clear();
        self.pending_bet = None;
        self.street = match self.street {
            Street::Preflop => Street::Flop,
            Street::Flop => Street::Turn,
            Street::Turn => Street::River,
            Street::River => Street::Showdown,
            s => s,
        };
        notes.push(format!("street -> {:?}", self.street));
    }

    /// Whose betting turn it is, derived purely from the log.
    pub fn to_act(&self) -> Option<Seat> {
        let betting = matches!(
            self.street,
            Street::Preflop | Street::Flop | Street::Turn | Street::River
        );
        // Betting on a postflop street only starts once its cards are public.
        let cards_in = match self.street {
            Street::Preflop => self.my_hole.len() == 2,
            Street::Flop => FLOP.iter().all(|s| self.public_cards.contains_key(s)),
            Street::Turn => self.public_cards.contains_key(&TURN),
            Street::River => self.public_cards.contains_key(&RIVER),
            _ => false,
        };
        if betting && cards_in && self.acts_this_street.len() < SEATS {
            Some(self.acts_this_street.len() as Seat)
        } else {
            None
        }
    }

    pub fn board_str(&self) -> String {
        let mut order: Vec<usize> = FLOP.to_vec();
        order.push(TURN);
        order.push(RIVER);
        order
            .iter()
            .filter_map(|s| self.public_cards.get(s))
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// What do I owe the protocol right now? Pure function of replayed state;
    /// `published` guards make each duty fire once.
    pub fn duties(&mut self) -> Vec<Duty> {
        let mut out = Vec::new();
        let me = self.me;

        // 1. Key share, in seat order so the log is deterministic.
        if self.joined.len() == SEATS
            && !self.keyshares.contains(&me)
            && (me == 0 || self.keyshares.contains(&(me - 1)))
            && self.mark(format!("key:{me}"))
        {
            out.push(Duty::Publish { kind: 'K', payload: me.to_string() });
        }

        // 2. Shuffle, in seat order. Seat 0 builds the initial masked deck.
        if self.keyshares.len() == SEATS && self.shuffles == me as usize && self.street == Street::Setup
        {
            if self.mark(format!("shuf:{me}")) {
                let base: Vec<PlainMasked> = if self.deck.is_empty() {
                    DECK_ARRAY
                        .iter()
                        .map(|&c| self.crypto.mask(&self.agg, &self.crypto.encode(c).unwrap()).0)
                        .collect()
                } else {
                    self.deck.clone()
                };
                let (next, _proof) = self.crypto.shuffle(&self.agg, &base);
                out.push(Duty::Publish { kind: 'S', payload: deck_to_str(&next) });
            }
        }

        // 3. Deal: publish my reveal token for every *other* seat's hole slots.
        if self.shuffles == SEATS {
            for (seat, slots) in HOLE_SLOTS.iter().enumerate() {
                if seat as Seat == me {
                    continue;
                }
                for &slot in slots {
                    if self.mark(format!("deal:{slot}")) {
                        out.push(self.reveal_duty(slot, &format!("{seat}")));
                    }
                }
            }
        }

        // 4. Board reveals for the current street (everyone publishes ToAll).
        let board_slots: &[usize] = match self.street {
            Street::Flop => &FLOP,
            Street::Turn => std::slice::from_ref(&TURN),
            Street::River => std::slice::from_ref(&RIVER),
            _ => &[],
        };
        for &slot in board_slots {
            if self.mark(format!("board:{slot}")) {
                out.push(self.reveal_duty(slot, "A"));
            }
        }

        // 5. Showdown: open my own hole cards to everyone.
        if self.street == Street::Showdown {
            for &slot in &HOLE_SLOTS[me as usize] {
                if self.mark(format!("show:{slot}")) {
                    out.push(self.reveal_duty(slot, "A"));
                }
            }
        }

        // 6. Betting turn.
        if self.to_act() == Some(me) {
            out.push(Duty::AwaitAction);
        }
        out
    }

    fn reveal_duty(&self, slot: usize, target: &str) -> Duty {
        let token = self.crypto.reveal_token(&self.me, &self.me, &self.deck[slot]);
        Duty::Publish {
            kind: 'R',
            payload: format!("{slot}|{target}|{}|{}", token.seat, token.card_ix),
        }
    }

    fn mark(&mut self, key: String) -> bool {
        self.published.insert(key)
    }
}
