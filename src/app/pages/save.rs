// SPDX-License-Identifier: GPL-3.0-only

//! The party and PC boxes of a Pokémon Platinum (Together) save, with the stats, ability and
//! moves the game really has, which randomizers make differ from the Pokédex.

use std::path::PathBuf;

use cosmic::iced::{Alignment, Background, Border, Color, Length};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokemon, DamageClass, SpriteKind, Stat};

use crate::app::pages::common::{card, fact, sprite, type_badge, type_icon};
use crate::app::pages::home::Section;
use crate::app::utils::gen4_save::{Gender, Member, Save, Trainer, nature_effect};
use crate::app::utils::presentation::{Label, data_lang, dex_number, nature_name, stat_short};
use crate::app::{AppModel, Message, MovesInput, Ready, SaveInput};
use crate::{fl, icons};

/// Width of the party list when it is shown next to the details.
const LIST_WIDTH: f32 = 300.0;
/// Narrower than this, the details are shown under the party list.
const SPLIT_MIN_WIDTH: f32 = 820.0;
const STATS: [Stat; 6] = [
    Stat::Hp,
    Stat::Attack,
    Stat::Defense,
    Stat::SpAttack,
    Stat::SpDefense,
    Stat::Speed,
];

/// Where a Pokémon of the save is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Position in the party
    Party(usize),
    /// Box number (0-based) and position in that box's list of Pokémon
    Box { index: usize, entry: usize },
    /// Position in the graveyard
    Graveyard(usize),
}

impl Default for Slot {
    fn default() -> Self {
        Self::Party(0)
    }
}

/// The save read from the game folder.
#[derive(Debug, Default)]
pub struct SaveState {
    /// File the save was read from
    pub path: Option<PathBuf>,
    pub data: Save,
    /// Why the save couldn't be read
    pub error: Option<String>,
    /// The save was found in the usual places, not in a folder the user picked
    pub detected: bool,
    /// Pokémon shown in the details
    pub selected: Slot,
}

impl SaveState {
    pub fn get(&self, slot: Slot) -> Option<&Member> {
        match slot {
            Slot::Party(i) => self.data.party.get(i),
            Slot::Box { index, entry } => self
                .data
                .boxes
                .get(index)
                .and_then(|b| b.pokemon.get(entry))
                .map(|(_, member)| member),
            Slot::Graveyard(i) => self.data.graveyard.get(i),
        }
    }

    /// The selected Pokémon, or the first one when the selection no longer exists.
    pub fn current(&self) -> Option<(Slot, &Member)> {
        let first_boxed = self
            .data
            .boxes
            .iter()
            .position(|b| !b.pokemon.is_empty())
            .map(|index| Slot::Box { index, entry: 0 });
        [
            Some(self.selected),
            Some(Slot::Party(0)),
            Some(Slot::Graveyard(0)),
            first_boxed,
        ]
        .into_iter()
        .flatten()
        .find_map(|slot| self.get(slot).map(|member| (slot, member)))
    }
}

pub fn view<'a>(app: &'a AppModel, ready: &'a Ready) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let save = &ready.save;
    let has_folder = !app.config.save_folder.is_empty() || save.path.is_some();

    let mut actions: Vec<Element<'a, Message>> = vec![
        text::title3(fl!("save-title")).into(),
        widget::Space::new().width(Length::Fill).into(),
    ];
    if has_folder {
        actions.push(
            button::text(fl!("save-reload"))
                .on_press(Message::Save(SaveInput::Reload))
                .into(),
        );
    }
    actions.push(
        button::standard(fl!("save-pick"))
            .on_press(Message::Save(SaveInput::PickFolder))
            .into(),
    );
    if !save.data.party.is_empty() {
        actions.push(
            widget::tooltip(
                button::suggested(fl!("save-use-as-team"))
                    .on_press(Message::Save(SaveInput::UseAsTeam)),
                text::body(fl!("save-use-as-team-hint")),
                widget::tooltip::Position::Bottom,
            )
            .into(),
        );
    }
    let header = widget::column::with_children(vec![
        widget::row::with_children(actions)
            .spacing(spacing.space_xs)
            .align_y(Alignment::Center)
            .into(),
        text::caption(match &save.path {
            Some(path) if save.detected => {
                fl!("save-source-detected", file = (path.display().to_string()))
            }
            Some(path) => fl!("save-source", file = (path.display().to_string())),
            None => fl!("save-hint"),
        })
        .into(),
    ])
    .spacing(4);

    let trainer = trainer_card(ready);

    let body: Element<'a, Message> = if let Some(error) = &save.error {
        card(fl!("save-error-title"), text::body(error.as_str()))
    } else if save.current().is_none() {
        cosmic::iced::widget::center(
            widget::column::with_children(vec![
                text::title4(fl!("save-empty-title")).into(),
                text::body(fl!("save-hint")).into(),
                button::suggested(fl!("save-pick"))
                    .on_press(Message::Save(SaveInput::PickFolder))
                    .into(),
            ])
            .spacing(12)
            .align_x(Alignment::Center),
        )
        .into()
    } else {
        widget::responsive(move |size| {
            let Some((slot, member)) = save.current() else {
                return widget::Space::new().into();
            };
            let details =
                widget::scrollable(container(member_details(ready, slot, member)).padding([
                    0,
                    spacing.space_m,
                    spacing.space_s,
                    0,
                ]))
                .height(Length::Fill)
                .width(Length::Fill);
            let list = widget::scrollable(container(pokemon_list(ready)).padding([
                0,
                spacing.space_s,
                0,
                0,
            ]));
            if size.width >= SPLIT_MIN_WIDTH {
                widget::row::with_children(vec![
                    list.width(Length::Fixed(LIST_WIDTH))
                        .height(Length::Fill)
                        .into(),
                    details.into(),
                ])
                .spacing(spacing.space_s)
                .into()
            } else {
                widget::column::with_children(vec![
                    list.height(Length::FillPortion(2)).into(),
                    details.height(Length::FillPortion(3)).into(),
                ])
                .spacing(spacing.space_s)
                .into()
            }
        })
        .into()
    };

    widget::column::with_children(vec![header.into(), trainer, body])
        .spacing(spacing.space_s)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Name shown for a party member: its nickname, or the species name.
fn display_name(pokemon: Option<&ArchivedPokemon>, member: &Member) -> String {
    member.nickname.clone().unwrap_or_else(|| {
        pokemon.map_or_else(
            || dex_number(u32::from(member.species)),
            |p| p.display.get(data_lang()).to_string(),
        )
    })
}

fn gender_symbol(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "♂",
        Gender::Female => "♀",
        Gender::Genderless => "",
    }
}

/// Level of a Pokémon: stored for the party, worked out from its experience in the boxes.
fn level(ready: &Ready, member: &Member) -> Option<u8> {
    member.battle.map(|b| b.level).or_else(|| {
        let dex = ready.core.dex();
        let species = dex.species_of(dex.pokemon(u32::from(member.species))?)?;
        Some(species.growth_rate.level_for(member.experience))
    })
}

fn level_label(ready: &Ready, member: &Member) -> String {
    level(ready, member).map_or_else(String::new, |level| fl!("save-level", level = level))
}

/// The party, then every box that holds a Pokémon. Boxed Pokémon are marked with a box icon.
fn pokemon_list<'a>(ready: &'a Ready) -> Element<'a, Message> {
    let save = &ready.save;
    let selected = save.current().map(|(slot, _)| slot);
    let mut children: Vec<Element<'a, Message>> = Vec::new();
    if !save.data.party.is_empty() {
        children.push(group_title("pokeball-symbolic", fl!("save-party")));
        children.extend(save.data.party.iter().enumerate().map(|(i, member)| {
            let slot = Slot::Party(i);
            pokemon_row(ready, slot, member, selected == Some(slot), None)
        }));
    }
    if !save.data.graveyard.is_empty() {
        children.push(group_title(
            "grave-symbolic",
            fl!("save-graveyard", count = save.data.graveyard.len()),
        ));
        children.extend(save.data.graveyard.iter().enumerate().map(|(i, member)| {
            let slot = Slot::Graveyard(i);
            let marker = ("grave-symbolic", fl!("save-in-graveyard"));
            pokemon_row(ready, slot, member, selected == Some(slot), Some(marker))
        }));
    }
    for (index, pc_box) in save.data.boxes.iter().enumerate() {
        if pc_box.pokemon.is_empty() {
            continue;
        }
        children.push(group_title(
            "pc-box-symbolic",
            fl!(
                "save-box-title",
                name = (pc_box.name.as_str()),
                count = pc_box.pokemon.len()
            ),
        ));
        children.extend(
            pc_box
                .pokemon
                .iter()
                .enumerate()
                .map(|(entry, (_, member))| {
                    let slot = Slot::Box { index, entry };
                    let marker = (
                        "pc-box-symbolic",
                        fl!("save-in-box", name = (pc_box.name.as_str())),
                    );
                    pokemon_row(ready, slot, member, selected == Some(slot), Some(marker))
                }),
        );
    }
    widget::column::with_children(children).spacing(2).into()
}

fn group_title<'a>(icon: &'static str, title: String) -> Element<'a, Message> {
    widget::row::with_children(vec![
        widget::icon(icons::get_handle(icon, 16)).size(16).into(),
        text::caption_heading(title).into(),
    ])
    .spacing(6)
    .align_y(Alignment::Center)
    .padding([10, 8, 4, 8])
    .into()
}

fn pokemon_row<'a>(
    ready: &'a Ready,
    slot: Slot,
    member: &'a Member,
    selected: bool,
    // Icon and tooltip telling the Pokémon isn't in the party
    marker: Option<(&'static str, String)>,
) -> Element<'a, Message> {
    let pokemon = ready.core.dex().pokemon(u32::from(member.species));
    let kind = SpriteKind::select(false, member.shiny, false);
    let picture = match pokemon {
        Some(p) => sprite(ready.core.sprite(p, kind), 44.0),
        None => widget::Space::new().width(Length::Fixed(44.0)).into(),
    };
    let mut row = vec![
        picture,
        widget::column::with_children(vec![
            text::heading(display_name(pokemon, member)).into(),
            text::caption(format!(
                "{} {}",
                level_label(ready, member),
                gender_symbol(member.gender)
            ))
            .into(),
        ])
        .width(Length::Fill)
        .into(),
    ];
    if let Some((icon, tooltip)) = marker {
        row.push(
            widget::tooltip(
                widget::icon(icons::get_handle(icon, 16)).size(16),
                text::body(tooltip),
                widget::tooltip::Position::Left,
            )
            .into(),
        );
    }
    button::custom(
        widget::row::with_children(row)
            .spacing(10)
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding([4, 8])
    .selected(selected)
    .class(theme::Button::IconVertical)
    .on_press(Message::Save(SaveInput::Select(slot)))
    .into()
}

/// Where the Pokémon is: in the party (accent), in a box (neutral) or in the graveyard (red),
/// so a boxed or fainted Pokémon isn't mistaken for a team member.
fn location_tag<'a>(ready: &'a Ready, slot: Slot) -> Element<'a, Message> {
    #[derive(Clone, Copy)]
    enum Tone {
        Accent,
        Neutral,
        Danger,
    }
    let (icon, label, tone) = match slot {
        Slot::Party(_) => ("pokeball-symbolic", fl!("save-in-party"), Tone::Accent),
        Slot::Graveyard(_) => ("grave-symbolic", fl!("save-in-graveyard"), Tone::Danger),
        Slot::Box { index, .. } => (
            "pc-box-symbolic",
            fl!(
                "save-in-box",
                name = (ready.save.data.boxes[index].name.as_str())
            ),
            Tone::Neutral,
        ),
    };
    container(
        widget::row::with_children(vec![
            widget::icon(icons::get_handle(icon, 16)).size(14).into(),
            text::caption_heading(label).into(),
        ])
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([3, 10])
    .class(theme::Container::custom(move |theme| {
        let cosmic = theme.cosmic();
        let tinted = |color: Color| (Color { a: 0.2, ..color }, color);
        let (background, foreground): (Color, Color) = match tone {
            Tone::Accent => tinted(cosmic.accent_color().into()),
            Tone::Danger => tinted(cosmic.destructive_color().into()),
            Tone::Neutral => (cosmic.bg_divider().into(), cosmic.on_bg_color().into()),
        };
        container::Style {
            background: Some(Background::Color(background)),
            text_color: Some(foreground),
            icon_color: Some(foreground),
            border: Border {
                radius: 12.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }))
    .into()
}

fn member_details<'a>(ready: &'a Ready, slot: Slot, member: &'a Member) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let dex = ready.core.dex();
    let lang = data_lang();
    let pokemon = dex.pokemon(u32::from(member.species));
    let name = display_name(pokemon, member);

    let mut identity = vec![
        text::caption(match pokemon {
            Some(p) if member.nickname.is_some() => {
                format!(
                    "{} · {}",
                    dex_number(u32::from(member.species)),
                    p.display.get(lang)
                )
            }
            _ => dex_number(u32::from(member.species)),
        })
        .class(theme::Text::Accent)
        .into(),
        text::title2(format!("{name} {}", gender_symbol(member.gender))).into(),
    ];
    let mut tags: Vec<Element<'a, Message>> = pokemon
        .map(|p| p.types.iter().map(|t| type_badge(*t)).collect())
        .unwrap_or_default();
    tags.push(text::body(level_label(ready, member)).into());
    if member.shiny {
        tags.push(text::body(fl!("shiny")).class(theme::Text::Accent).into());
    }
    identity.push(
        widget::row::with_children(tags)
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
    );

    identity.push(location_tag(ready, slot));

    let mut links: Vec<Element<'a, Message>> = Vec::new();
    if let Some(p) = pokemon {
        let id = p.id.to_native();
        links.push(
            button::standard(fl!("save-open-pokedex"))
                .on_press(Message::SelectPokemon(id))
                .into(),
        );
        links.push(
            button::standard(fl!("compare-with"))
                .on_press(Message::CompareWith(id))
                .into(),
        );
    }

    let kind = SpriteKind::select(false, member.shiny, member.gender == Gender::Female);
    let header = widget::row::with_children(vec![
        match pokemon {
            Some(p) => sprite(ready.core.sprite(p, kind), 112.0),
            None => widget::Space::new().width(Length::Fixed(112.0)).into(),
        },
        widget::column::with_children(identity)
            .spacing(4)
            .width(Length::Fill)
            .into(),
        widget::column::with_children(links)
            .spacing(6)
            .align_x(Alignment::End)
            .into(),
    ])
    .spacing(spacing.space_s)
    .align_y(Alignment::Center);

    let nature = match nature_effect(member.nature) {
        Some((up, down)) => format!(
            "{} (+{} −{})",
            nature_name(member.nature),
            stat_short(STATS[up]),
            stat_short(STATS[down])
        ),
        None => nature_name(member.nature),
    };
    let ability = dex
        .ability(u32::from(member.ability))
        .map_or_else(|| fl!("unknown"), |a| a.display.get(lang).to_string());
    let facts = widget::row::with_children(vec![
        fact(fl!("save-nature"), nature),
        fact(fl!("save-ability"), ability),
        fact(fl!("save-friendship"), member.friendship.to_string()),
        fact(fl!("save-experience"), member.experience.to_string()),
        fact(fl!("save-trainer"), member.original_trainer.clone()),
    ])
    .spacing(spacing.space_xs);

    widget::column::with_children(vec![
        header.into(),
        facts.into(),
        stats_card(member, matches!(slot, Slot::Graveyard(_))),
        moves_card(ready, member),
        text::caption(fl!("save-randomizer-note")).into(),
    ])
    .spacing(spacing.space_s)
    .into()
}

/// Real stats with IVs and EVs; the stats raised and lowered by the nature are marked.
/// Boxed Pokémon don't store their stats (the game works them out when they leave the box),
/// so only their IVs and EVs are shown.
fn stats_card<'a>(member: &'a Member, in_graveyard: bool) -> Element<'a, Message> {
    let effect = nature_effect(member.nature);
    let highest = member.battle.map_or(1.0, |b| {
        f32::from(*b.stats.iter().max().unwrap_or(&1)).max(1.0)
    });
    let mut rows: Vec<Element<'a, Message>> = vec![
        widget::row::with_children(vec![
            widget::Space::new().width(Length::Fixed(64.0)).into(),
            text::caption_heading(fl!("save-value"))
                .width(Length::Fixed(72.0))
                .align_x(Alignment::End)
                .into(),
            widget::Space::new().width(Length::Fill).into(),
            text::caption_heading(fl!("save-iv"))
                .width(Length::Fixed(40.0))
                .align_x(Alignment::End)
                .into(),
            text::caption_heading(fl!("save-ev"))
                .width(Length::Fixed(40.0))
                .align_x(Alignment::End)
                .into(),
        ])
        .spacing(10)
        .into(),
    ];
    for (i, stat) in STATS.iter().enumerate() {
        let mut label = stat_short(*stat);
        let mut accent = false;
        match effect {
            Some((up, _)) if up == i => {
                label.push_str(" ↑");
                accent = true;
            }
            Some((_, down)) if down == i => label.push_str(" ↓"),
            _ => {}
        }
        let label = text::body(label).width(Length::Fixed(64.0));
        let (value_text, bar): (String, Element<'a, Message>) = match member.battle {
            Some(battle) => {
                let value = battle.stats[i];
                (
                    if i == 0 {
                        format!("{}/{value}", battle.current_hp)
                    } else {
                        value.to_string()
                    },
                    widget::progress_bar::determinate_linear(f32::from(value) / highest)
                        .width(Length::Fill)
                        .girth(8)
                        .into(),
                )
            }
            None => (
                "—".to_string(),
                widget::Space::new().width(Length::Fill).into(),
            ),
        };
        rows.push(
            widget::row::with_children(vec![
                if accent {
                    label.class(theme::Text::Accent).into()
                } else {
                    label.into()
                },
                text::heading(value_text)
                    .width(Length::Fixed(72.0))
                    .align_x(Alignment::End)
                    .into(),
                bar,
                text::body(member.ivs[i].to_string())
                    .width(Length::Fixed(40.0))
                    .align_x(Alignment::End)
                    .into(),
                text::body(member.evs[i].to_string())
                    .width(Length::Fixed(40.0))
                    .align_x(Alignment::End)
                    .into(),
            ])
            .spacing(10)
            .align_y(Alignment::Center)
            .into(),
        );
    }
    if member.battle.is_none() {
        rows.push(
            text::caption(if in_graveyard {
                fl!("save-graveyard-stats")
            } else {
                fl!("save-boxed-stats")
            })
            .into(),
        );
    }
    card(
        fl!("save-stats"),
        widget::column::with_children(rows).spacing(8),
    )
}

/// The four moves with their PP; clicking one opens its details.
fn moves_card<'a>(ready: &'a Ready, member: &'a Member) -> Element<'a, Message> {
    let dex = ready.core.dex();
    let lang = data_lang();
    let rows: Vec<Element<'a, Message>> = member
        .moves
        .iter()
        .map(|slot| match dex.move_(u32::from(slot.id)) {
            Some(attack) => {
                let max_pp = attack
                    .pp
                    .as_ref()
                    .map(|pp| u16::from(*pp) * (5 + u16::from(slot.pp_ups)) / 5);
                let power = attack
                    .power
                    .as_ref()
                    .filter(|_| attack.damage_class != DamageClass::Status)
                    .map_or_else(|| "—".to_string(), u8::to_string);
                let row = widget::row::with_children(vec![
                    type_icon(attack.move_type, 18),
                    text::heading(attack.display.get(lang))
                        .width(Length::Fill)
                        .into(),
                    text::caption(attack.damage_class.label())
                        .width(Length::Fixed(80.0))
                        .into(),
                    text::caption(power)
                        .width(Length::Fixed(40.0))
                        .align_x(Alignment::End)
                        .into(),
                    text::caption(match max_pp {
                        Some(max) => fl!("save-pp", pp = slot.pp, max = max),
                        None => fl!("save-pp-left", pp = slot.pp),
                    })
                    .width(Length::Fixed(80.0))
                    .align_x(Alignment::End)
                    .into(),
                ])
                .spacing(10)
                .align_y(Alignment::Center);
                button::custom(row)
                    .width(Length::Fill)
                    .padding([6, 8])
                    .class(theme::Button::IconVertical)
                    .on_press(Message::Moves(MovesInput::Open(attack.id.to_native())))
                    .into()
            }
            None => text::body(fl!("save-unknown-move", id = slot.id)).into(),
        })
        .collect();
    card(
        fl!("save-moves"),
        widget::column::with_children(rows).spacing(2),
    )
}

/// Link from the team page to this section.
pub fn link<'a>() -> Element<'a, Message> {
    button::standard(fl!("save-from-game"))
        .on_press(Message::ShowSection(Section::Save))
        .into()
}

/// Trainer card, Pokédex progress and how many Pokémon are in the graveyard.
fn trainer_card<'a>(ready: &'a Ready) -> Element<'a, Message> {
    let data = &ready.save.data;
    let Some(trainer) = &data.trainer else {
        return widget::Space::new().into();
    };
    let time = trainer.play_time;
    let mut tiles = vec![
        fact(
            fl!("save-trainer"),
            format!(
                "{} {}",
                trainer.name,
                if trainer.female { "♀" } else { "♂" }
            ),
        ),
        fact(fl!("save-trainer-id"), format!("{:05}", trainer.id)),
        fact(
            fl!("save-money"),
            fl!("item-cost-value", cost = trainer.money),
        ),
        fact(
            fl!("save-badges"),
            format!("{}/{}", trainer.badges, Trainer::BADGES),
        ),
        fact(
            fl!("save-play-time"),
            format!("{}:{:02}", time.hours, time.minutes),
        ),
    ];
    if let Some(dex) = &data.pokedex {
        tiles.push(fact(fl!("save-caught"), dex.caught.len().to_string()));
        tiles.push(fact(fl!("save-seen"), dex.seen.len().to_string()));
    }
    tiles.push(fact(
        fl!("save-graveyard-title"),
        data.graveyard.len().to_string(),
    ));
    widget::flex_row(
        tiles
            .into_iter()
            .map(|tile| container(tile).width(Length::Fixed(150.0)).into())
            .collect(),
    )
    .row_spacing(8)
    .column_spacing(8)
    .into()
}
