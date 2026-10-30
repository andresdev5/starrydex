// SPDX-License-Identifier: GPL-3.0-only

//! Reads a Pokémon Platinum (Gen 4) save, like the ones Platinum Together writes: trainer,
//! Pokédex progress, party, PC boxes, and the extra boxes Platinum Together adds (one of them
//! is the graveyard, where fainted Pokémon end up for good). The file is only read, never
//! modified.

use std::path::{Path, PathBuf};

/// Size of a Platinum save file.
const SAVE_SIZE: usize = 0x80000;
/// The general block is stored twice; the copy with the higher save counter is the newest.
const COPIES: [usize; 2] = [0x0, 0x40000];
const GENERAL_BLOCK_SIZE: usize = 0xCF2C;
const FOOTER_MAGIC: u32 = 0x2006_0623;
/// Storage block (PC boxes), right after the general block and its footer.
const STORAGE_START: usize = 0xCF40;
const STORAGE_SIZE: usize = 0x121EC;
/// The boxes come after the current box number and another word.
const BOX_DATA: usize = STORAGE_START + 8;
const BOX_COUNT: usize = 18;
const BOX_SLOTS: usize = 30;
/// A boxed Pokémon has no battle stats.
const BOXED_SIZE: usize = 136;
/// Box names (20 characters each) follow the boxes.
const BOX_NAMES: usize = BOX_DATA + BOX_COUNT * BOX_SLOTS * BOXED_SIZE;
const BOX_NAME_SIZE: usize = 40;
/// Extra boxes added by Platinum Together, with their own header and two copies.
const EXTRA_BOXES: usize = 0x2C000;
const EXTRA_MAGIC: &[u8; 4] = b"PTBX";
/// Header: magic, version (u16), box count (u16), save counter (u32), padding.
const EXTRA_HEADER_SIZE: usize = 0x10;
/// Each extra box name is followed by other settings.
const EXTRA_NAME_SIZE: usize = 40;
const EXTRA_NAME_CHARS: usize = 20;
/// Name of the extra box Platinum Together uses as the graveyard.
const GRAVEYARD: &str = "GRAVEYARD";
/// Trainer data inside the general block.
const TRAINER: usize = 0x74;
/// Pokédex inside the general block: a magic number, then caught and seen flags.
const POKEDEX: usize = 0x1334;
const POKEDEX_MAGIC: u32 = 0xBEEF_CAFE;
const POKEDEX_FLAGS_SIZE: usize = 0x40;
/// Possible places of the party size inside the general block; the party follows it. Platinum
/// Together saves use the first one.
const PARTY_COUNT_OFFSETS: [usize; 3] = [0xA8, 0xA0, 0x9C];
const PARTY_SIZE: usize = 6;
/// A party Pokémon: 136 bytes of boxed data plus 100 bytes of battle stats.
const SLOT_SIZE: usize = 236;
/// Block stored at each of the four 32-byte positions (0 = A: species, 1 = B: moves...),
/// for each shuffle value.
const BLOCK_ORDERS: [[u8; 4]; 24] = [
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];
/// Highest national dex number in generation 4.
const LAST_SPECIES: u16 = 493;

/// Stats in the order the app uses: HP, Attack, Defense, Sp. Atk, Sp. Def, Speed.
pub type StatSpread<T> = [T; 6];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
    Genderless,
}

/// A move slot of a Pokémon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveSlot {
    /// Move id (same numbering as PokéAPI)
    pub id: u16,
    /// PP left
    pub pp: u8,
    /// PP Ups used (each adds a fifth of the base PP)
    pub pp_ups: u8,
}

/// What the game computes for Pokémon in the party; boxed Pokémon don't store it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleStats {
    pub level: u8,
    pub current_hp: u16,
    pub stats: StatSpread<u16>,
}

/// A Pokémon as stored in the save (randomizers change its stats, ability and moves, so they
/// may not match the Pokédex).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// National dex number
    pub species: u16,
    /// Only set when the player gave it a nickname
    pub nickname: Option<String>,
    /// 0–24, in the games' order (Hardy, Lonely, Brave...)
    pub nature: u8,
    /// Ability id (same numbering as PokéAPI)
    pub ability: u8,
    pub gender: Gender,
    pub shiny: bool,
    pub friendship: u8,
    pub experience: u32,
    pub original_trainer: String,
    /// Only for Pokémon in the party
    pub battle: Option<BattleStats>,
    pub ivs: StatSpread<u8>,
    pub evs: StatSpread<u8>,
    pub moves: Vec<MoveSlot>,
}

/// The stats a nature raises and lowers, as indices into a [`StatSpread`]; `None` for the
/// neutral natures.
pub fn nature_effect(nature: u8) -> Option<(usize, usize)> {
    // Natures cycle through Attack, Defense, Speed, Sp. Atk, Sp. Def.
    const STATS: [usize; 5] = [1, 2, 5, 3, 4];
    let (up, down) = (usize::from(nature / 5), usize::from(nature % 5));
    (up != down && up < 5).then(|| (STATS[up], STATS[down]))
}

/// A PC box with the slots (0–29) that hold a Pokémon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcBox {
    pub name: String,
    pub pokemon: Vec<(usize, Member)>,
}

/// Party and PC boxes of a save.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Save {
    pub trainer: Option<Trainer>,
    pub pokedex: Option<PokedexProgress>,
    pub party: Vec<Member>,
    /// Every box (the 18 of the game, then the extra ones), empty ones included
    pub boxes: Vec<PcBox>,
    /// Fainted Pokémon (Platinum Together's graveyard box)
    pub graveyard: Vec<Member>,
}

/// Trainer card data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trainer {
    pub name: String,
    pub female: bool,
    /// Public trainer ID
    pub id: u16,
    pub money: u32,
    /// Badges earned, out of [`Trainer::BADGES`]
    pub badges: u32,
    pub play_time: PlayTime,
}

impl Trainer {
    /// Badges of the Sinnoh region.
    pub const BADGES: u32 = 8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayTime {
    pub hours: u16,
    pub minutes: u8,
    pub seconds: u8,
}

/// National dex numbers of the species caught and seen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PokedexProgress {
    pub caught: Vec<u16>,
    pub seen: Vec<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveError {
    /// Not a Platinum save (wrong size or no valid block)
    NotASave,
}

/// The save file inside `folder`, which can be the Platinum Together install (with its Wine
/// prefix), its `Game` folder, or any folder with a `save.bin`.
pub fn find_save(folder: &Path) -> Option<PathBuf> {
    let direct = [
        folder.join("save.bin"),
        folder.join("Game").join("save.bin"),
        folder
            .join("PlatinumTogether")
            .join("Game")
            .join("save.bin"),
    ];
    if let Some(save) = direct.into_iter().find(|p| p.is_file()) {
        return Some(save);
    }
    let users = folder.join("prefix").join("drive_c").join("users");
    std::fs::read_dir(users)
        .ok()?
        .filter_map(Result::ok)
        .map(|user| {
            user.path()
                .join("AppData/Local/PlatinumTogether/Game/save.bin")
        })
        .find(|p| p.is_file())
}

/// The party and the PC boxes, each from its newest copy.
pub fn read_save(save: &[u8]) -> Result<Save, SaveError> {
    if save.len() < SAVE_SIZE {
        return Err(SaveError::NotASave);
    }
    let general = newest_copy(save, GENERAL_BLOCK_SIZE).ok_or(SaveError::NotASave)?;
    let mut boxes = newest_copy(save, STORAGE_START + STORAGE_SIZE - 0x14)
        .map(|base| read_boxes(save, base))
        .unwrap_or_default();
    let mut graveyard = Vec::new();
    for extra in read_extra_boxes(save) {
        if extra.name.eq_ignore_ascii_case(GRAVEYARD) {
            graveyard.extend(extra.pokemon.into_iter().map(|(_, member)| member));
        } else {
            boxes.push(extra);
        }
    }
    Ok(Save {
        trainer: Some(read_trainer(save, general)),
        pokedex: read_pokedex(save, general),
        party: read_party(save, general),
        boxes,
        graveyard,
    })
}

fn read_trainer(save: &[u8], base: usize) -> Trainer {
    let t = base + TRAINER;
    Trainer {
        name: decode_text(&save[t..t + 16]),
        id: u16_at(save, t + 0x10),
        money: u32_at(save, t + 0x14),
        female: save[t + 0x18] == 1,
        badges: u32::from(save[t + 0x1A]).count_ones(),
        play_time: PlayTime {
            hours: u16_at(save, t + 0x22),
            minutes: save[t + 0x24],
            seconds: save[t + 0x25],
        },
    }
}

fn read_pokedex(save: &[u8], base: usize) -> Option<PokedexProgress> {
    let dex = base + POKEDEX;
    if u32_at(save, dex) != POKEDEX_MAGIC {
        return None;
    }
    let species = |flags: usize| -> Vec<u16> {
        (1..=LAST_SPECIES)
            .filter(|n| {
                let bit = usize::from(n - 1);
                save[flags + bit / 8] & (1 << (bit % 8)) != 0
            })
            .collect()
    };
    Some(PokedexProgress {
        caught: species(dex + 4),
        seen: species(dex + 4 + POKEDEX_FLAGS_SIZE),
    })
}

/// Platinum Together's extra boxes (none in other saves), from the newest copy.
fn read_extra_boxes(save: &[u8]) -> Vec<PcBox> {
    let Some(base) = COPIES
        .iter()
        .map(|copy| copy + EXTRA_BOXES)
        .filter(|&header| &save[header..header + 4] == EXTRA_MAGIC)
        .max_by_key(|&header| u32_at(save, header + 8))
    else {
        return Vec::new();
    };
    let count = usize::from(u16_at(save, base + 6));
    let slots = base + EXTRA_HEADER_SIZE;
    let names = slots + count * BOX_SLOTS * BOXED_SIZE;
    if names + count * EXTRA_NAME_SIZE > save.len() {
        return Vec::new();
    }
    (0..count)
        .map(|index| {
            let name = names + index * EXTRA_NAME_SIZE;
            PcBox {
                name: decode_text(&save[name..name + EXTRA_NAME_CHARS]),
                pokemon: (0..BOX_SLOTS)
                    .filter_map(|slot| {
                        let start = slots + (index * BOX_SLOTS + slot) * BOXED_SIZE;
                        decode(&save[start..start + BOXED_SIZE]).map(|member| (slot, member))
                    })
                    .collect(),
            }
        })
        .collect()
}

/// Start of the copy (of the block whose footer is at `footer`) with the highest save counter.
fn newest_copy(save: &[u8], footer: usize) -> Option<usize> {
    COPIES
        .iter()
        .copied()
        .filter(|&base| u32_at(save, base + footer + 0xC) == FOOTER_MAGIC)
        .max_by_key(|&base| u32_at(save, base + footer))
}

fn read_party(save: &[u8], base: usize) -> Vec<Member> {
    for offset in PARTY_COUNT_OFFSETS {
        let count = u32_at(save, base + offset) as usize;
        if !(1..=PARTY_SIZE).contains(&count) {
            continue;
        }
        let party: Option<Vec<Member>> = (0..count)
            .map(|i| {
                let start = base + offset + 4 + i * SLOT_SIZE;
                decode(&save[start..start + SLOT_SIZE])
            })
            .collect();
        if let Some(party) = party {
            return party;
        }
    }
    Vec::new()
}

fn read_boxes(save: &[u8], base: usize) -> Vec<PcBox> {
    (0..BOX_COUNT)
        .map(|index| {
            let name = base + BOX_NAMES + index * BOX_NAME_SIZE;
            PcBox {
                name: decode_text(&save[name..name + BOX_NAME_SIZE]),
                pokemon: (0..BOX_SLOTS)
                    .filter_map(|slot| {
                        let start = base + BOX_DATA + (index * BOX_SLOTS + slot) * BOXED_SIZE;
                        decode(&save[start..start + BOXED_SIZE]).map(|member| (slot, member))
                    })
                    .collect(),
            }
        })
        .collect()
}

/// Decrypts a party (236 bytes) or boxed (136 bytes) Pokémon; `None` for empty slots and
/// when the checksum doesn't match.
fn decode(slot: &[u8]) -> Option<Member> {
    let pid = u32_at(slot, 0);
    let checksum = u16_at(slot, 6);
    if pid == 0 && checksum == 0 {
        return None;
    }
    let blocks = crypt(&slot[8..BOXED_SIZE], u32::from(checksum));
    let sum = (0..blocks.len())
        .step_by(2)
        .fold(0u16, |sum, i| sum.wrapping_add(u16_at(&blocks, i)));
    if sum != checksum {
        return None;
    }
    let order = BLOCK_ORDERS[(((pid & 0x3E000) >> 0xD) % 24) as usize];
    let block = |letter: u8| -> &[u8] {
        let position = order.iter().position(|&b| b == letter).unwrap_or(0);
        &blocks[position * 32..position * 32 + 32]
    };
    let (a, b, c, d) = (block(0), block(1), block(2), block(3));

    let species = u16_at(a, 0x00);
    if !(1..=LAST_SPECIES).contains(&species) {
        return None;
    }
    let trainer_id = u16_at(a, 0x04);
    let secret_id = u16_at(a, 0x06);
    // Stored as HP, Attack, Defense, Speed, Sp. Atk, Sp. Def.
    let game_order = |values: [u8; 6]| {
        [
            values[0], values[1], values[2], values[4], values[5], values[3],
        ]
    };
    let evs = game_order([a[0x10], a[0x11], a[0x12], a[0x13], a[0x14], a[0x15]]);
    let iv_word = u32_at(b, 0x10);
    let ivs = game_order(std::array::from_fn(|i| ((iv_word >> (5 * i)) & 0x1F) as u8));
    let flags = b[0x18];
    let gender = if flags & 0x04 != 0 {
        Gender::Genderless
    } else if flags & 0x02 != 0 {
        Gender::Female
    } else {
        Gender::Male
    };
    let moves = (0..4)
        .filter_map(|i| {
            let id = u16_at(b, 2 * i);
            (id != 0).then(|| MoveSlot {
                id,
                pp: b[0x08 + i],
                pp_ups: b[0x0C + i],
            })
        })
        .collect();

    let battle = (slot.len() >= SLOT_SIZE).then(|| {
        let battle = crypt(&slot[BOXED_SIZE..SLOT_SIZE], pid);
        let stat = |offset: usize| u16_at(&battle, offset);
        BattleStats {
            level: battle[0x04],
            current_hp: stat(0x06),
            // Stored as HP, Attack, Defense, Speed, Sp. Atk, Sp. Def.
            stats: [
                stat(0x08),
                stat(0x0A),
                stat(0x0C),
                stat(0x10),
                stat(0x12),
                stat(0x0E),
            ],
        }
    });
    Some(Member {
        species,
        nickname: (iv_word & (1 << 31) != 0).then(|| decode_text(&c[..22])),
        nature: (pid % 25) as u8,
        ability: a[0x0D],
        gender,
        shiny: (u32::from(trainer_id ^ secret_id) ^ (pid >> 16) ^ (pid & 0xFFFF)) < 8,
        friendship: a[0x0C],
        experience: u32_at(a, 0x08),
        original_trainer: decode_text(&d[..16]),
        battle,
        ivs,
        evs,
        moves,
    })
}

/// Decodes a name in the Gen 4 character set: digits, Latin letters and a few symbols;
/// other characters are left out.
fn decode_text(data: &[u8]) -> String {
    let mut text = String::new();
    for i in (0..data.len()).step_by(2) {
        let code = u16_at(data, i);
        let c = match code {
            0xFFFF => break,
            0x0121..=0x012A => char::from(b'0' + (code - 0x0121) as u8),
            0x012B..=0x0144 => char::from(b'A' + (code - 0x012B) as u8),
            0x0145..=0x015E => char::from(b'a' + (code - 0x0145) as u8),
            0x01AB => '!',
            0x01AC => '?',
            0x01AD => ',',
            0x01AE => '.',
            0x01B3 => '\'',
            0x01BB => '♂',
            0x01BC => '♀',
            0x01BE => '-',
            0x01DE => ' ',
            _ => continue,
        };
        text.push(c);
    }
    text
}

/// XORs every 16-bit word with the Gen 4 pseudo-random sequence started at `seed`
/// (the same operation encrypts and decrypts).
fn crypt(data: &[u8], mut seed: u32) -> Vec<u8> {
    let mut out = data.to_vec();
    for word in out.as_chunks_mut::<2>().0 {
        seed = seed.wrapping_mul(0x41C6_4E6D).wrapping_add(0x6073);
        *word = (u16::from_le_bytes(*word) ^ (seed >> 16) as u16).to_le_bytes();
    }
    out
}

fn u16_at(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn u32_at(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the test puts in a party slot.
    struct Spec {
        pid: u32,
        species: u16,
        level: u8,
        nickname: Option<&'static str>,
        moves: [u16; 4],
    }

    fn encode_text(text: &str, words: usize) -> Vec<u8> {
        let mut out: Vec<u16> = text
            .chars()
            .map(|c| match c {
                '0'..='9' => 0x0121 + (c as u16 - '0' as u16),
                'A'..='Z' => 0x012B + (c as u16 - 'A' as u16),
                'a'..='z' => 0x0145 + (c as u16 - 'a' as u16),
                _ => 0x01DE,
            })
            .collect();
        out.push(0xFFFF);
        out.resize(words, 0);
        out.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    /// Encrypts a party Pokémon the way the game stores it.
    fn encode(spec: &Spec) -> Vec<u8> {
        let mut a = vec![0u8; 32];
        a[0x00..0x02].copy_from_slice(&spec.species.to_le_bytes());
        a[0x08..0x0C].copy_from_slice(&1000u32.to_le_bytes());
        a[0x0C] = 70;
        a[0x0D] = 65;
        // EVs as HP, Atk, Def, Spe, SpA, SpD
        a[0x10..0x16].copy_from_slice(&[1, 2, 3, 6, 4, 5]);
        let mut b = vec![0u8; 32];
        for (i, id) in spec.moves.iter().enumerate() {
            b[2 * i..2 * i + 2].copy_from_slice(&id.to_le_bytes());
            b[0x08 + i] = 10 + i as u8;
        }
        // IVs as HP, Atk, Def, Spe, SpA, SpD = 10, 11, 12, 15, 13, 14
        let mut ivs = [10u32, 11, 12, 15, 13, 14]
            .iter()
            .enumerate()
            .fold(0, |word, (i, iv)| word | (iv << (5 * i)));
        if spec.nickname.is_some() {
            ivs |= 1 << 31;
        }
        b[0x10..0x14].copy_from_slice(&ivs.to_le_bytes());
        b[0x18] = 0x02;
        let c = encode_text(spec.nickname.unwrap_or(""), 16);
        let d = encode_text("white", 16);

        let order = BLOCK_ORDERS[(((spec.pid & 0x3E000) >> 0xD) % 24) as usize];
        let by_letter = [a, b, c, d];
        let blocks: Vec<u8> = order
            .iter()
            .flat_map(|&letter| by_letter[letter as usize].clone())
            .collect();
        let checksum = (0..128)
            .step_by(2)
            .fold(0u16, |sum, i| sum.wrapping_add(u16_at(&blocks, i)));

        let mut battle = vec![0u8; 100];
        battle[4] = spec.level;
        // Current HP, then max HP, Atk, Def, Spe, SpA, SpD
        for (i, value) in [40u16, 45, 20, 21, 24, 22, 23].iter().enumerate() {
            battle[6 + 2 * i..8 + 2 * i].copy_from_slice(&value.to_le_bytes());
        }

        let mut slot = Vec::with_capacity(SLOT_SIZE);
        slot.extend_from_slice(&spec.pid.to_le_bytes());
        slot.extend_from_slice(&[0, 0]);
        slot.extend_from_slice(&checksum.to_le_bytes());
        slot.extend(crypt(&blocks, u32::from(checksum)));
        slot.extend(crypt(&battle, spec.pid));
        slot
    }

    fn save_with(party: &[Spec], counters: [u32; 2]) -> Vec<u8> {
        let mut save = vec![0u8; SAVE_SIZE];
        for (copy, base) in COPIES.into_iter().enumerate() {
            let footer = base + GENERAL_BLOCK_SIZE;
            save[footer..footer + 4].copy_from_slice(&counters[copy].to_le_bytes());
            save[footer + 0xC..footer + 0x10].copy_from_slice(&FOOTER_MAGIC.to_le_bytes());
        }
        // Only the newest copy gets the party.
        let base = if counters[0] >= counters[1] {
            COPIES[0]
        } else {
            COPIES[1]
        };
        let count = base + PARTY_COUNT_OFFSETS[0];
        save[count..count + 4].copy_from_slice(&(party.len() as u32).to_le_bytes());
        for (i, spec) in party.iter().enumerate() {
            let start = count + 4 + i * SLOT_SIZE;
            save[start..start + SLOT_SIZE].copy_from_slice(&encode(spec));
        }
        save
    }

    fn buizel() -> Spec {
        Spec {
            pid: 0x1234_5678,
            species: 418,
            level: 14,
            nickname: None,
            moves: [253, 228, 0, 0],
        }
    }

    #[test]
    fn block_orders_follow_the_gen4_table() {
        let table = "ABCD ABDC ACBD ACDB ADBC ADCB BACD BADC BCAD BCDA BDAC BDCA \
                     CABD CADB CBAD CBDA CDAB CDBA DABC DACB DBAC DBCA DCAB DCBA";
        let letters: Vec<String> = BLOCK_ORDERS
            .iter()
            .map(|order| order.iter().map(|&b| char::from(b'A' + b)).collect())
            .collect();
        assert_eq!(
            letters.join(" "),
            table.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }

    #[test]
    fn reads_the_party_of_the_newest_copy() {
        let houndour = Spec {
            pid: 0xDEAD_BEEF,
            species: 228,
            level: 5,
            nickname: Some("Rex"),
            moves: [435, 0, 0, 0],
        };
        let party = read_save(&save_with(&[buizel(), houndour], [5, 6]))
            .unwrap()
            .party;
        assert_eq!(party.len(), 2);
        let level = |m: &Member| m.battle.map(|b| b.level);
        assert_eq!((party[0].species, level(&party[0])), (418, Some(14)));
        assert_eq!((party[1].species, level(&party[1])), (228, Some(5)));
        assert_eq!(party[0].nickname, None);
        assert_eq!(party[1].nickname.as_deref(), Some("Rex"));
    }

    #[test]
    fn reads_stats_and_moves_in_app_order() {
        let party = read_save(&save_with(&[buizel()], [1, 0])).unwrap().party;
        let member = &party[0];
        let battle = member.battle.unwrap();
        assert_eq!(battle.current_hp, 40);
        assert_eq!(battle.stats, [45, 20, 21, 22, 23, 24]);
        assert_eq!(member.ivs, [10, 11, 12, 13, 14, 15]);
        assert_eq!(member.evs, [1, 2, 3, 4, 5, 6]);
        assert_eq!(
            member.moves,
            vec![
                MoveSlot {
                    id: 253,
                    pp: 10,
                    pp_ups: 0
                },
                MoveSlot {
                    id: 228,
                    pp: 11,
                    pp_ups: 0
                },
            ]
        );
        assert_eq!(member.original_trainer, "white");
        assert_eq!(
            (member.ability, member.friendship, member.experience),
            (65, 70, 1000)
        );
        assert_eq!(member.gender, Gender::Female);
        assert_eq!(member.nature, (0x1234_5678u32 % 25) as u8);
    }

    #[test]
    fn natures_raise_and_lower_stats() {
        // Hardy is neutral, Lonely (+Atk −Def), Modest (+SpA −Atk), Timid (+Spe −Atk)
        assert_eq!(nature_effect(0), None);
        assert_eq!(nature_effect(1), Some((1, 2)));
        assert_eq!(nature_effect(15), Some((3, 1)));
        assert_eq!(nature_effect(10), Some((5, 1)));
    }

    #[test]
    fn rejects_other_files() {
        assert_eq!(read_save(&[0; 1024]), Err(SaveError::NotASave));
        assert_eq!(read_save(&vec![0; SAVE_SIZE]), Err(SaveError::NotASave));
    }

    #[test]
    fn reads_an_empty_party() {
        assert_eq!(read_save(&save_with(&[], [1, 0])).unwrap().party, vec![]);
    }

    #[test]
    fn rejects_corrupted_pokemon() {
        let mut save = save_with(&[buizel()], [1, 0]);
        let slot = PARTY_COUNT_OFFSETS[0] + 4;
        save[slot + 20] ^= 0xFF;
        assert_eq!(read_save(&save).unwrap().party, vec![]);
    }

    #[test]
    fn reads_the_trainer_and_the_pokedex() {
        let mut save = save_with(&[], [1, 0]);
        let t = TRAINER;
        save[t..t + 16].copy_from_slice(&encode_text("white", 8));
        save[t + 0x10..t + 0x12].copy_from_slice(&28980u16.to_le_bytes());
        save[t + 0x14..t + 0x18].copy_from_slice(&1956u32.to_le_bytes());
        save[t + 0x18] = 1;
        save[t + 0x1A] = 0b0000_0101;
        save[t + 0x22..t + 0x24].copy_from_slice(&2u16.to_le_bytes());
        save[t + 0x24] = 48;
        save[t + 0x25] = 2;
        save[POKEDEX..POKEDEX + 4].copy_from_slice(&POKEDEX_MAGIC.to_le_bytes());
        // Caught: #1 and #9; seen: #1, #9 and #493
        save[POKEDEX + 4] = 0b1;
        save[POKEDEX + 5] = 0b1;
        let seen = POKEDEX + 4 + POKEDEX_FLAGS_SIZE;
        save[seen] = 0b1;
        save[seen + 1] = 0b1;
        save[seen + 61] = 0b1_0000;

        let parsed = read_save(&save).unwrap();
        assert_eq!(
            parsed.trainer,
            Some(Trainer {
                name: "white".into(),
                female: true,
                id: 28980,
                money: 1956,
                badges: 2,
                play_time: PlayTime {
                    hours: 2,
                    minutes: 48,
                    seconds: 2
                },
            })
        );
        let dex = parsed.pokedex.unwrap();
        assert_eq!(dex.caught, vec![1, 9]);
        assert_eq!(dex.seen, vec![1, 9, 493]);
    }

    #[test]
    fn puts_the_graveyard_apart_from_the_extra_boxes() {
        let mut save = save_with(&[], [1, 0]);
        // Two copies of the extra boxes; the second one has the higher counter.
        for (copy, counter) in COPIES.into_iter().zip([17u32, 18]) {
            let header = copy + EXTRA_BOXES;
            save[header..header + 4].copy_from_slice(EXTRA_MAGIC);
            save[header + 4..header + 6].copy_from_slice(&1u16.to_le_bytes());
            save[header + 6..header + 8].copy_from_slice(&2u16.to_le_bytes());
            save[header + 8..header + 12].copy_from_slice(&counter.to_le_bytes());
            let names = header + EXTRA_HEADER_SIZE + 2 * BOX_SLOTS * BOXED_SIZE;
            save[names..names + EXTRA_NAME_CHARS].copy_from_slice(&encode_text("BOX 19", 10));
            let graveyard = names + EXTRA_NAME_SIZE;
            save[graveyard..graveyard + EXTRA_NAME_CHARS]
                .copy_from_slice(&encode_text("GRAVEYARD", 10));
        }
        let slot = COPIES[1] + EXTRA_BOXES + EXTRA_HEADER_SIZE + BOX_SLOTS * BOXED_SIZE;
        save[slot..slot + BOXED_SIZE].copy_from_slice(&encode(&buizel())[..BOXED_SIZE]);

        let parsed = read_save(&save).unwrap();
        assert_eq!(parsed.boxes.len(), 1);
        assert_eq!(parsed.boxes[0].name, "BOX 19");
        assert_eq!(parsed.graveyard.len(), 1);
        assert_eq!(parsed.graveyard[0].species, 418);
    }

    #[test]
    fn reads_the_boxes_of_the_newest_storage_copy() {
        let mut save = save_with(&[], [1, 0]);
        let footer = STORAGE_START + STORAGE_SIZE - 0x14;
        for (copy, counter) in COPIES.into_iter().zip([3u32, 4]) {
            save[copy + footer..copy + footer + 4].copy_from_slice(&counter.to_le_bytes());
            save[copy + footer + 0xC..copy + footer + 0x10]
                .copy_from_slice(&FOOTER_MAGIC.to_le_bytes());
        }
        // The second copy is newer: box 2, slot 5 holds a Pokémon there.
        let base = COPIES[1];
        let slot = base + BOX_DATA + (BOX_SLOTS + 4) * BOXED_SIZE;
        save[slot..slot + BOXED_SIZE].copy_from_slice(&encode(&buizel())[..BOXED_SIZE]);
        let name = base + BOX_NAMES + BOX_NAME_SIZE;
        save[name..name + BOX_NAME_SIZE].copy_from_slice(&encode_text("Water", 20));

        let boxes = read_save(&save).unwrap().boxes;
        assert_eq!(boxes.len(), BOX_COUNT);
        assert!(boxes[0].pokemon.is_empty());
        assert_eq!(boxes[1].name, "Water");
        let (position, member) = &boxes[1].pokemon[0];
        assert_eq!((*position, member.species), (4, 418));
        assert_eq!(member.battle, None);
        assert_eq!(member.moves.len(), 2);
    }
}
