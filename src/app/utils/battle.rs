// SPDX-License-Identifier: GPL-3.0-only

//! Rough battle math for the comparator: stats and damage at level 50 with perfect IVs, no
//! EVs, a neutral nature, no items or abilities, and no random roll or critical hits.

/// Level the comparator assumes for both Pokémon.
pub const LEVEL: u16 = 50;
const PERFECT_IV: u16 = 31;
const STAB: f32 = 1.5;

/// Battle stat at [`LEVEL`] from a base stat.
pub fn stat_at_level(base: u8, is_hp: bool) -> u16 {
    let core = (2 * u16::from(base) + PERFECT_IV) * LEVEL / 100;
    if is_hp { core + LEVEL + 10 } else { core + 5 }
}

/// Damage of one hit, following the main-series formula (highest roll).
pub fn damage(power: u8, attack: u16, defense: u16, stab: bool, multiplier: f32) -> u16 {
    let level_factor = u32::from(2 * LEVEL / 5 + 2);
    let base = level_factor * u32::from(power) * u32::from(attack) / u32::from(defense.max(1));
    let mut damage = (base / 50 + 2) as f32;
    if stab {
        damage = (damage * STAB).floor();
    }
    (damage * multiplier).floor() as u16
}

/// Hits needed to knock out a Pokémon with `hp`, or `None` when the hit does no damage.
pub fn hits_to_ko(hp: u16, damage: u16) -> Option<u16> {
    (damage > 0).then(|| hp.div_ceil(damage))
}

/// How many turns a move takes, which the data set doesn't say.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pace {
    Normal,
    /// Spends a turn charging (or out of reach) before hitting
    Charge,
    /// Must rest a turn after hitting
    Recharge,
    /// Knocks the user out
    SelfKo,
}

impl Pace {
    /// Pace of the move with the given PokéAPI name.
    pub fn of(move_name: &str) -> Self {
        match move_name {
            "solar-beam" | "solar-blade" | "sky-attack" | "skull-bash" | "razor-wind"
            | "freeze-shock" | "ice-burn" | "meteor-beam" | "electro-shot" | "dig" | "fly"
            | "dive" | "bounce" | "phantom-force" | "shadow-force" | "sky-drop" => Self::Charge,
            "hyper-beam" | "giga-impact" | "blast-burn" | "hydro-cannon" | "frenzy-plant"
            | "rock-wrecker" | "roar-of-time" | "eternabeam" | "prismatic-laser"
            | "meteor-assault" => Self::Recharge,
            "explosion" | "self-destruct" | "misty-explosion" | "memento" | "final-gambit" => {
                Self::SelfKo
            }
            _ => Self::Normal,
        }
    }

    /// Turns needed to land `hits` hits (a recharge after the last one doesn't matter).
    pub fn turns(self, hits: u16) -> u16 {
        match self {
            Self::Normal | Self::SelfKo => hits,
            Self::Charge => 2 * hits,
            Self::Recharge => (2 * hits).saturating_sub(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_at_level_50() {
        // Charizard: 78 HP, 84 Atk, 109 SpA
        assert_eq!(stat_at_level(78, true), 153);
        assert_eq!(stat_at_level(84, false), 104);
        assert_eq!(stat_at_level(109, false), 129);
    }

    #[test]
    fn damage_follows_the_formula() {
        // Charizard's Flamethrower (90, special, STAB) into Blastoise (Sp. Def 125), resisted
        assert_eq!(damage(90, 129, 125, true, 0.5), 31);
        // Same hit without STAB nor resistance
        assert_eq!(damage(90, 129, 125, false, 1.0), 42);
        assert_eq!(damage(90, 129, 125, true, 0.0), 0);
    }

    #[test]
    fn slow_moves_take_more_turns() {
        assert_eq!(Pace::of("flamethrower").turns(3), 3);
        assert_eq!(Pace::of("solar-beam").turns(2), 4);
        assert_eq!(Pace::of("hydro-cannon").turns(1), 1);
        assert_eq!(Pace::of("hyper-beam").turns(2), 3);
        assert_eq!(Pace::of("explosion"), Pace::SelfKo);
    }

    #[test]
    fn hits_round_up() {
        assert_eq!(hits_to_ko(153, 31), Some(5));
        assert_eq!(hits_to_ko(150, 50), Some(3));
        assert_eq!(hits_to_ko(150, 0), None);
    }
}
