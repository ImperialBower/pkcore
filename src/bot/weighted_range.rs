//! Frequency-weighted hand range entries.
//!
//! [`WeightedRange`] represents a mixed strategy as an ordered list of
//! combo-string/frequency pairs. Each [`ComboWeight`] names a range token
//! (using the same notation as [`RangeStrategy`](crate::bot::range_strategy::RangeStrategy),
//! e.g. `"AJs+"`, `"66+"`, `"KQs"`) and the frequency with which that range
//! is played (0.0 = never, 1.0 = always).
//!
//! Use [`WeightedRange::from_flat`] to convert an existing flat range string
//! (all combos at frequency 1.0) and [`WeightedRange::push`] to build a
//! mixed-strategy range entry by entry.

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, Visitor},
};
use std::fmt;

// ── ComboWeight ───────────────────────────────────────────────────────────────

/// A single hand range token paired with a mixed-strategy frequency.
///
/// `range` uses the standard combo-string notation already used by
/// [`RangeStrategy`](crate::bot::range_strategy::RangeStrategy):
/// e.g. `"AKs"`, `"QQ+"`, `"JJ-TT"`, `"54s+"`.
///
/// `frequency` is clamped to `[0.0, 1.0]` on construction.
///
/// Serializes as a compact string `"AKs:0.8"` — no spaces.
///
/// # Examples
///
/// ```
/// use pkcore::bot::weighted_range::ComboWeight;
///
/// let cw = ComboWeight::new("AQs", 0.8);
/// assert_eq!(cw.range, "AQs");
/// assert_eq!(cw.frequency, 0.8);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ComboWeight {
    /// The range token, e.g. `"AJs+"` or `"66+"`.
    pub range: String,
    /// How often this range is played: `0.0` = never, `1.0` = always.
    pub frequency: f64,
}

impl Serialize for ComboWeight {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if (self.frequency - 1.0).abs() < f64::EPSILON {
            return self.range.serialize(s);
        }
        let freq = if self.frequency.fract() == 0.0 {
            format!("{:.1}", self.frequency)
        } else {
            format!("{}", self.frequency)
        };
        format!("{}:{}", self.range, freq).serialize(s)
    }
}

impl<'de> Deserialize<'de> for ComboWeight {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct ComboWeightVisitor;

        impl<'de> Visitor<'de> for ComboWeightVisitor {
            type Value = ComboWeight;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a string \"AKs:1.0\" or a single-entry map {{\"AKs\": 1.0}}")
            }

            // string form: "AKs" (implicit 1.0), "AKs:0.75", or "AKs: 0.75"
            fn visit_str<E: de::Error>(self, v: &str) -> Result<ComboWeight, E> {
                match v.rsplit_once(':') {
                    None => Ok(ComboWeight::new(v, 1.0)),
                    Some((range, freq_str)) => {
                        let frequency = freq_str
                            .trim()
                            .parse::<f64>()
                            .map_err(|e| E::custom(format!("invalid frequency: {e}")))?;
                        Ok(ComboWeight::new(range, frequency))
                    }
                }
            }

            // single-entry map form: {AKs: 1.0}
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<ComboWeight, A::Error> {
                let (range, frequency) = map
                    .next_entry::<String, f64>()?
                    .ok_or_else(|| de::Error::custom("expected one entry"))?;
                Ok(ComboWeight::new(range, frequency))
            }
        }

        d.deserialize_any(ComboWeightVisitor)
    }
}

impl ComboWeight {
    /// Creates a new [`ComboWeight`], clamping `frequency` to `[0.0, 1.0]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::ComboWeight;
    ///
    /// let cw = ComboWeight::new("KQs", 1.5); // clamped to 1.0
    /// assert_eq!(cw.frequency, 1.0);
    /// ```
    #[must_use]
    pub fn new(range: impl Into<String>, frequency: f64) -> Self {
        Self {
            range: range.into(),
            frequency: frequency.clamp(0.0, 1.0),
        }
    }
}

// ── WeightedRange ─────────────────────────────────────────────────────────────

/// An ordered list of [`ComboWeight`] entries representing one action's range.
///
/// Range tokens follow the same comma-separated notation used by
/// [`RangeStrategy`](crate::bot::range_strategy::RangeStrategy). Use
/// [`WeightedRange::from_flat`] to convert a flat range string (all entries
/// at frequency 1.0), or build a mixed strategy with [`WeightedRange::push`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::weighted_range::WeightedRange;
///
/// let wr = WeightedRange::from_flat("AA,KK,QQ");
/// assert_eq!(wr.frequency_for("AA"), 1.0);
/// assert_eq!(wr.frequency_for("22"), 0.0);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WeightedRange {
    combos: Vec<ComboWeight>,
}

impl WeightedRange {
    /// Creates an empty [`WeightedRange`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// let wr = WeightedRange::new();
    /// assert!(wr.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a [`WeightedRange`] from a flat comma-separated range string,
    /// assigning frequency `1.0` to every token.
    ///
    /// Tokens are trimmed of whitespace. Empty tokens (e.g. from trailing
    /// commas) are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// let wr = WeightedRange::from_flat("TT+, AQs+, KQs");
    /// assert_eq!(wr.frequency_for("TT+"), 1.0);
    /// assert_eq!(wr.frequency_for("AQs+"), 1.0);
    /// assert_eq!(wr.frequency_for("KQs"), 1.0);
    /// assert_eq!(wr.frequency_for("22"), 0.0);
    /// ```
    #[must_use]
    pub fn from_flat(range_str: &str) -> Self {
        let mut wr = Self::new();
        for token in range_str.split(',') {
            let token = token.trim();
            if !token.is_empty() {
                wr.push(token, 1.0);
            }
        }
        wr
    }

    /// Appends a combo token at the given frequency, clamped to `[0.0, 1.0]`.
    ///
    /// Returns `&mut Self` for chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// let mut wr = WeightedRange::new();
    /// wr.push("AA", 1.0).push("KK", 0.75);
    /// assert_eq!(wr.len(), 2);
    /// ```
    pub fn push(&mut self, range: impl Into<String>, frequency: f64) -> &mut Self {
        self.combos.push(ComboWeight::new(range, frequency));
        self
    }

    /// Returns the frequency for the first entry whose range string exactly
    /// matches `combo`, or `0.0` if not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// let mut wr = WeightedRange::new();
    /// wr.push("AQs+", 0.8);
    /// assert_eq!(wr.frequency_for("AQs+"), 0.8);
    /// assert_eq!(wr.frequency_for("AQs"), 0.0);  // not an exact match
    /// ```
    #[must_use]
    pub fn frequency_for(&self, combo: &str) -> f64 {
        self.combos
            .iter()
            .find(|cw| cw.range == combo)
            .map_or(0.0, |cw| cw.frequency)
    }

    /// Returns a slice of all [`ComboWeight`] entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// let wr = WeightedRange::from_flat("AA,KK");
    /// assert_eq!(wr.combos().len(), 2);
    /// ```
    #[must_use]
    pub fn combos(&self) -> &[ComboWeight] {
        &self.combos
    }

    /// Returns the number of entries in this range.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// assert_eq!(WeightedRange::new().len(), 0);
    /// assert_eq!(WeightedRange::from_flat("AA,KK").len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.combos.len()
    }

    /// Returns `true` if this range contains no entries.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// assert!(WeightedRange::new().is_empty());
    /// assert!(!WeightedRange::from_flat("AA").is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.combos.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combo_weight_new_clamps_above_one() {
        let cw = ComboWeight::new("AA", 1.5);
        assert_eq!(cw.frequency, 1.0);
    }

    #[test]
    fn test_combo_weight_new_clamps_below_zero() {
        let cw = ComboWeight::new("AA", -0.3);
        assert_eq!(cw.frequency, 0.0);
    }

    #[test]
    fn test_weighted_range_new_is_empty() {
        assert!(WeightedRange::new().is_empty());
    }

    #[test]
    fn test_weighted_range_from_flat_all_at_one() {
        let wr = WeightedRange::from_flat("AA,KK,QQ");
        assert_eq!(wr.len(), 3);
        assert_eq!(wr.frequency_for("AA"), 1.0);
        assert_eq!(wr.frequency_for("KK"), 1.0);
        assert_eq!(wr.frequency_for("QQ"), 1.0);
    }

    #[test]
    fn test_weighted_range_from_flat_trims_whitespace() {
        let wr = WeightedRange::from_flat("TT+, AQs+, KQs");
        assert_eq!(wr.len(), 3);
        assert_eq!(wr.frequency_for("TT+"), 1.0);
        assert_eq!(wr.frequency_for("AQs+"), 1.0);
        assert_eq!(wr.frequency_for("KQs"), 1.0);
    }

    #[test]
    fn test_weighted_range_from_flat_ignores_empty_tokens() {
        let wr = WeightedRange::from_flat("AA,,KK,");
        assert_eq!(wr.len(), 2);
    }

    #[test]
    fn test_weighted_range_frequency_for_unknown_returns_zero() {
        let wr = WeightedRange::from_flat("AA,KK");
        assert_eq!(wr.frequency_for("22"), 0.0);
    }

    #[test]
    fn test_weighted_range_frequency_for_exact_match_only() {
        let mut wr = WeightedRange::new();
        wr.push("AJs+", 0.8);
        assert_eq!(wr.frequency_for("AJs+"), 0.8);
        assert_eq!(wr.frequency_for("AJs"), 0.0); // not an exact match
    }

    #[test]
    fn test_weighted_range_push_chaining() {
        let mut wr = WeightedRange::new();
        wr.push("AA", 1.0).push("KK", 0.75).push("QQ", 0.5);
        assert_eq!(wr.len(), 3);
        assert_eq!(wr.frequency_for("KK"), 0.75);
    }

    #[test]
    fn test_weighted_range_serde_round_trip() {
        let mut wr = WeightedRange::new();
        wr.push("AA", 1.0).push("KK", 0.8);
        let json = serde_json::to_string(&wr).unwrap();
        let loaded: WeightedRange = serde_json::from_str(&json).unwrap();
        assert_eq!(wr, loaded);
    }

    #[test]
    fn test_combo_weight_deserialize_bare_string_is_full_frequency() {
        let cw: ComboWeight = serde_yaml_bw::from_str("AA").unwrap();
        assert_eq!(cw.range, "AA");
        assert_eq!(cw.frequency, 1.0);
    }

    #[test]
    fn test_combo_weight_deserialize_compact_with_frequency() {
        let cw: ComboWeight = serde_yaml_bw::from_str("AKs:0.75").unwrap();
        assert_eq!(cw.range, "AKs");
        assert_eq!(cw.frequency, 0.75);
    }

    #[test]
    fn test_combo_weight_deserialize_map_with_space() {
        let cw: ComboWeight = serde_yaml_bw::from_str("AA: 0.75").unwrap();
        assert_eq!(cw.range, "AA");
        assert_eq!(cw.frequency, 0.75);
    }

    #[test]
    fn test_combo_weight_serialize_full_frequency_is_bare() {
        let yaml = serde_yaml_bw::to_string(&ComboWeight::new("AKs", 1.0)).unwrap();
        assert!(yaml.trim() == "AKs", "expected bare string, got: {yaml}");
    }

    #[test]
    fn test_combo_weight_serialize_partial_frequency_has_value() {
        let yaml = serde_yaml_bw::to_string(&ComboWeight::new("AKs", 0.75)).unwrap();
        assert!(yaml.contains("AKs:0.75"), "expected compact form, got: {yaml}");
    }
}
