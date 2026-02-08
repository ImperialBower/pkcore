use crate::prelude::Seats;

#[derive(Clone, Debug)]
pub struct PotInfo {
    pub amount: usize,
    pub eligible_seats: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct PotManager {
    pub pots: Vec<PotInfo>,
}

impl PotManager {
    #[must_use]
    pub fn create_pots(seats: &Seats) -> Self {
        let mut contributions: Vec<(u8, usize)> = seats
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_in_hand())
            .map(|(i, s)| {
                (
                    u8::try_from(i).unwrap_or_default(),
                    s.borrow().player.chips_in_play.get(),
                )
            })
            .collect();

        contributions.sort_by_key(|(_, amt)| *amt);

        let mut pots = Vec::new();
        let mut prev_amount = 0;

        while !contributions.is_empty() {
            let total_contrib = contributions[0].1;

            if total_contrib > prev_amount {
                let pot_size = (total_contrib - prev_amount) * contributions.len();
                let eligible = contributions.iter().map(|(s, _)| *s).collect();

                pots.push(PotInfo {
                    amount: pot_size,
                    eligible_seats: eligible,
                });

                prev_amount = total_contrib;
            }

            contributions.retain(|(_, amt)| *amt > total_contrib);
        }

        PotManager { pots }
    }
}
