//! Game-state-derived estimates of what active villains hold (EPIC-39).
//!
//! The decider models villains as [`PlayerSpec::Random`] by default. This
//! module narrows that to a [`Combos`] range derived from **position alone** —
//! never from opponent identity — so the equity engine can sample realistic
//! holdings instead of any two cards.
//!
//! [`PlayerSpec::Random`]: crate::analysis::equity::PlayerSpec::Random

use crate::analysis::gto::combos::Combos;
use crate::bot::profile::BotProfile;
use crate::bot::table_snapshot::TableSnapshot;
use crate::bot::weighted_range::WeightedRange;
use crate::casino::position::Position;
use std::str::FromStr;

/// The action key every villain range is looked up under.
///
/// A [`TableSnapshot`] carries no event log, so postflop the decider cannot
/// tell who raised preflop and who merely called. Every villain therefore gets
/// its position's opening range, which overstates a caller's strength. See the
/// EPIC-39 corrigendum.
const VILLAIN_ACTION: &str = "open_raise";

/// Flattens a [`WeightedRange`] into a [`Combos`] set.
///
/// Entries at frequency `0.0` are dropped; everything else is kept at full
/// weight, because [`PlayerSpec::Range`] samples its combinations *uniformly*
/// and has no way to honour a mixed frequency. Returns `None` when nothing
/// survives or the notation does not parse.
///
/// [`PlayerSpec::Range`]: crate::analysis::equity::PlayerSpec::Range
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::twos::Twos;
/// use pkcore::bot::range_model::combos_from_weighted;
/// use pkcore::bot::weighted_range::WeightedRange;
///
/// let combos = combos_from_weighted(&WeightedRange::from_flat("QQ+")).unwrap();
/// // `Combos` stores notation, so "QQ+" is one combo carrying a `plus` flag.
/// assert_eq!(1, combos.len());
/// // It expands to the eighteen hands QQ, KK and AA only when asked.
/// assert_eq!(18, Twos::from(&combos).to_vec().len());
///
/// assert!(combos_from_weighted(&WeightedRange::new()).is_none());
/// ```
#[must_use]
pub fn combos_from_weighted(range: &WeightedRange) -> Option<Combos> {
    let notation = range
        .combos()
        .iter()
        .filter(|weighted| weighted.frequency > 0.0)
        .map(|weighted| weighted.range.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if notation.is_empty() {
        return None;
    }
    let combos = Combos::from_str(&notation).ok()?;
    if combos.is_empty() { None } else { Some(combos) }
}

/// Estimates the range the villain at `villain_index` is representing.
///
/// `villain_index` indexes [`TableSnapshot::stacks`], which holds the occupied
/// seats in seat order — so the index *is* the logical seat, and the villain's
/// [`Position`] follows from the button. The range itself comes from the
/// hero profile's shared position data, never from who the opponent is.
///
/// Returns `None` when the index is out of range, names the hero, names a seat
/// that is no longer active, or when no position or range can be resolved.
///
/// # Examples
///
/// ```
/// use pkcore::bot::playbook::Playbook;
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::range_model::villain_range;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(
///     (0..6)
///         .map(|i| Seat::new(Player::new_with_chips(format!("P{i}"), 1_000)))
///         .collect(),
/// );
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let state = TableSnapshot::from_table(&table, 3);
/// let profile = BotProfile::gto().with_playbook(Playbook::gto());
///
/// // Seat 0 holds the button and opens a wide range.
/// assert!(villain_range(&profile, &state, 0).is_some());
/// // The hero is not a villain.
/// assert!(villain_range(&profile, &state, 3).is_none());
/// ```
#[must_use]
pub fn villain_range(profile: &BotProfile, state: &TableSnapshot, villain_index: usize) -> Option<Combos> {
    let villain = state.stacks.get(villain_index)?;
    if villain.seat == state.seat || !villain.is_active {
        return None;
    }
    let logical = u8::try_from(villain_index).ok()?;
    let position = Position::from_seat(logical, state.dealer_button?, state.seat_count)?;
    let range = profile.range_for_or_default(state.seat_count, position, VILLAIN_ACTION);
    combos_from_weighted(&range)
}

/// Builds one [`PlayerSpec`] per active villain, in seat order.
///
/// Villains become [`PlayerSpec::Range`] only when the profile opts in with
/// [`RangeMode::PositionAware`] *and* a range resolves for that seat; anything
/// else stays [`PlayerSpec::Random`], which is the pre-EPIC-39 behaviour. A
/// range that cannot be resolved degrades to `Random` rather than to an empty
/// range, because the equity engine rejects a range with no live combination.
///
/// [`PlayerSpec`]: crate::analysis::equity::PlayerSpec
/// [`PlayerSpec::Range`]: crate::analysis::equity::PlayerSpec::Range
/// [`PlayerSpec::Random`]: crate::analysis::equity::PlayerSpec::Random
/// [`RangeMode::PositionAware`]: crate::bot::decision_config::RangeMode::PositionAware
///
/// # Examples
///
/// ```
/// use pkcore::analysis::equity::PlayerSpec;
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::range_model::villain_specs;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
/// ]);
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let state = TableSnapshot::from_table(&table, 0);
///
/// // The default profile keeps the historical Random villain.
/// let specs = villain_specs(&BotProfile::gto(), &state);
/// assert_eq!(1, specs.len());
/// assert!(matches!(specs[0], PlayerSpec::Random));
/// ```
#[cfg(feature = "equity")]
#[must_use]
pub fn villain_specs(profile: &BotProfile, state: &TableSnapshot) -> Vec<crate::analysis::equity::PlayerSpec> {
    use crate::analysis::equity::PlayerSpec;
    use crate::bot::decision_config::RangeMode;

    if matches!(profile.decision.ranges, RangeMode::PositionAware) {
        return villain_range_specs(profile, state);
    }
    state
        .stacks
        .iter()
        .filter(|villain| villain.is_active && villain.seat != state.seat)
        .map(|_| PlayerSpec::Random)
        .collect()
}

/// Builds one [`PlayerSpec`] per active villain, always preferring a range.
///
/// This is [`villain_specs`] without the `ranges` knob gate, for callers whose
/// own knob already means "model villains as ranges" — the `preflop_charts`
/// solver path is the one in tree. A seat whose range cannot be resolved is
/// still [`PlayerSpec::Random`].
///
/// [`PlayerSpec`]: crate::analysis::equity::PlayerSpec
/// [`PlayerSpec::Random`]: crate::analysis::equity::PlayerSpec::Random
///
/// # Examples
///
/// ```
/// use pkcore::analysis::equity::PlayerSpec;
/// use pkcore::bot::playbook::Playbook;
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::range_model::villain_range_specs;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(
///     (0..6)
///         .map(|i| Seat::new(Player::new_with_chips(format!("P{i}"), 1_000)))
///         .collect(),
/// );
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let state = TableSnapshot::from_table(&table, 3);
/// let profile = BotProfile::gto().with_playbook(Playbook::gto());
///
/// // Ranged even though the profile is on the default `ranges: flat`.
/// let specs = villain_range_specs(&profile, &state);
/// assert_eq!(5, specs.len());
/// assert!(specs.iter().all(|spec| matches!(spec, PlayerSpec::Range(_))));
/// ```
#[cfg(feature = "equity")]
#[must_use]
pub fn villain_range_specs(profile: &BotProfile, state: &TableSnapshot) -> Vec<crate::analysis::equity::PlayerSpec> {
    use crate::analysis::equity::PlayerSpec;

    state
        .stacks
        .iter()
        .enumerate()
        .filter(|(_, villain)| villain.is_active && villain.seat != state.seat)
        .map(|(index, _)| villain_range(profile, state, index).map_or(PlayerSpec::Random, PlayerSpec::Range))
        .collect()
}

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__range_model_tests {
    use super::*;
    use crate::analysis::gto::twos::Twos;
    use crate::arrays::two::Two;
    use crate::bot::playbook::Playbook;
    use crate::casino::game::ForcedBets;
    use crate::casino::table::{Player, Seat, Seats, Table};

    fn six_max_snapshot(hero_seat: u8) -> TableSnapshot<'static> {
        let seats = Seats::new(
            (0..6)
                .map(|i| Seat::new(Player::new_with_chips(format!("P{i}"), 1_000)))
                .collect(),
        );
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        TableSnapshot::from_table(&table, hero_seat)
    }

    fn gto_profile() -> BotProfile {
        BotProfile::gto().with_playbook(Playbook::gto())
    }

    fn hands(combos: &Combos) -> Vec<Two> {
        Twos::from(combos).to_vec()
    }

    #[test]
    fn combos_from_weighted_parses_a_flat_range() {
        let range = WeightedRange::from_flat("QQ+");
        let combos = combos_from_weighted(&range).expect("QQ+ is a parseable range");
        assert_eq!(18, hands(&combos).len(), "QQ+ is QQ, KK and AA — 18 hands");
    }

    #[test]
    fn combos_from_weighted_is_none_for_an_empty_range() {
        assert!(combos_from_weighted(&WeightedRange::new()).is_none());
    }

    #[test]
    fn combos_from_weighted_drops_zero_frequency_entries() {
        let mut range = WeightedRange::new();
        range.push("AA", 1.0);
        range.push("72o", 0.0);
        let combos = combos_from_weighted(&range).expect("AA survives");
        assert_eq!(6, hands(&combos).len(), "only the six AA hands remain");
    }

    #[test]
    fn villain_range_on_the_button_is_wider_than_in_the_cutoff() {
        let profile = gto_profile();
        let state = six_max_snapshot(3); // hero is LJ, so index 0 is BTN and index 5 is CO
        let btn = villain_range(&profile, &state, 0).expect("the BTN villain has a range");
        let cutoff = villain_range(&profile, &state, 5).expect("the CO villain has a range");
        assert!(
            hands(&btn).len() > hands(&cutoff).len(),
            "BTN opens wider than CO: {} vs {}",
            hands(&btn).len(),
            hands(&cutoff).len()
        );
    }

    #[test]
    fn villain_range_on_the_button_includes_a_weak_suited_ace() {
        let profile = gto_profile();
        let state = six_max_snapshot(3);
        let btn = villain_range(&profile, &state, 0).expect("the BTN villain has a range");
        let a2s = Two::from_str("AS 2S").expect("AS 2S parses");
        assert!(hands(&btn).contains(&a2s), "the BTN open_raise range covers A2s");
    }

    #[test]
    fn villain_range_is_none_for_the_hero_seat() {
        let profile = gto_profile();
        let state = six_max_snapshot(3);
        assert!(
            villain_range(&profile, &state, 3).is_none(),
            "the hero is not a villain"
        );
    }

    #[test]
    fn villain_range_is_none_for_an_index_past_the_table() {
        let profile = gto_profile();
        let state = six_max_snapshot(0);
        assert!(villain_range(&profile, &state, 99).is_none());
    }
    #[cfg(feature = "equity")]
    #[test]
    fn villain_specs_are_random_for_a_default_profile() {
        use crate::analysis::equity::PlayerSpec;
        let state = six_max_snapshot(3);
        let specs = villain_specs(&gto_profile(), &state);
        assert_eq!(5, specs.len(), "five villains at a full six-max table");
        assert!(
            specs.iter().all(|spec| matches!(spec, PlayerSpec::Random)),
            "a profile on the default `ranges: flat` keeps today's Random villains"
        );
    }

    #[cfg(feature = "equity")]
    #[test]
    fn villain_specs_use_ranges_when_position_aware() {
        use crate::analysis::equity::PlayerSpec;
        use crate::bot::decision_config::RangeMode;
        let mut profile = gto_profile();
        profile.decision.ranges = RangeMode::PositionAware;
        let state = six_max_snapshot(3);
        let specs = villain_specs(&profile, &state);
        assert_eq!(5, specs.len());
        assert!(
            specs.iter().all(|spec| matches!(spec, PlayerSpec::Range(_))),
            "every seated villain resolves to a position range"
        );
    }

    #[cfg(feature = "equity")]
    #[test]
    fn villain_specs_fall_back_to_random_without_a_playbook() {
        use crate::analysis::equity::PlayerSpec;
        use crate::bot::decision_config::RangeMode;
        let mut profile = BotProfile::gto();
        profile.playbook = None;
        profile.range_strategy.open_raise = String::new();
        profile.decision.ranges = RangeMode::PositionAware;
        let state = six_max_snapshot(3);
        let specs = villain_specs(&profile, &state);
        assert!(
            specs.iter().all(|spec| matches!(spec, PlayerSpec::Random)),
            "an unresolvable range must degrade to Random, never to an empty range"
        );
    }
}
