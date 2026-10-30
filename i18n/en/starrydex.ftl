## App

app-title = StarryDex
loading = Loading...

# Header menu ("View" menu)
view = View
about = About
settings = Settings

# About page
app-info = This application has been made for learning purposes.
pokeapi-text = This application uses PokéApi and its resources.
nintendo-text = Pokémon and Pokémon character names are trademarks of Nintendo.
repository = Repository
support = Support


## Settings page

# Appearance section
appearance = Appearance
theme = Theme
match-desktop = Match desktop
dark = Dark
light = Light
modern-dex = Modern Dex
pokemon-per-page = Pokémon per page

# Pokédex section
pokedex-section = Pokédex
show-forms = Show alternate forms
show-forms-description = List megas, regional forms and other varieties as separate entries
cry-volume = Cry volume
type-filter-mode = Type filtering mode
exclusive = Exclusive
inclusive = Inclusive

# Other section (cache and bundled data)
other = Other
renew-cache = Renew Cache
renew-cache-description = Re-extracts the bundled Pokédex data, sprites and cries
renew-cache-button = Renew
data-info = Data: { $pokemon } Pokémon, { $species } species, { $moves } moves · generated { $date }


## Main page: sections

home-pokemon = Pokémon
home-types = Types
home-items = Items
home-moves = Moves
moves-search-placeholder = Search moves by name
moves-all-classes = All classes
moves-all-generations = All generations
moves-range = { $start }–{ $end } of { $total } moves
moves-none = No moves
moves-no-results = No moves found
moves-no-results-hint = Try another search or remove some filters.
moves-type = Type
moves-class = Class
moves-generation = Gen.
moves-machines = Taught by machine
moves-learned-by = Learned by ({ $count })
move-details = Move
home-compare = Compare
home-team = Team
home-team-count = Team ({ $count }/6)
favorite-add = Add to favorites
favorite-remove = Remove from favorites
favorites-only = Only favorites
team-add = Add to team
team-remove = Remove from team
team-full = The team is full
compare-with = Compare
team-title = My team
team-count = { $count } of 6 members
team-add-slot = + Add from the Pokédex
home-save = Game
save-title = Game save
save-pick = Pick game folder…
save-reload = Read again
save-use-as-team = Use as my team
save-use-as-team-hint = Replaces your team with the party of the save.
save-from-game = From the game save
save-dialog = Pick the game folder
save-hint = Pick the Pokémon Platinum Together (or Platinum) folder to see the party as the game has it: real stats, ability and moves, even with a randomizer. The save is only read, never changed.
save-source = Read from { $file }
save-empty-title = No game save yet
save-error-title = Couldn't read the save
save-not-found = No Platinum save (save.bin) was found in { $folder }.
save-not-a-save = { $file } is not a Pokémon Platinum save.
save-read-error = Couldn't read { $file }: { $error }
save-party = Party
save-box-title = { $name } · { $count }
save-in-party = In the party
save-in-box = In box “{ $name }”
save-graveyard-stats = Pokémon in the graveyard don't store their stats, only their IVs and EVs.
save-boxed-stats = Pokémon in the PC don't store their stats: the game works them out when they leave the box. IVs and EVs are shown.
save-graveyard = Graveyard · { $count }
save-graveyard-title = Graveyard
save-in-graveyard = In the graveyard
save-trainer-id = ID No.
save-money = Money
save-badges = Badges
save-play-time = Play time
save-caught = Caught
save-seen = Seen
save-level = Lv. { $level }
save-open-pokedex = See in Pokédex
save-nature = Nature
save-ability = Ability
save-friendship = Friendship
save-experience = Experience
save-trainer = Trainer
save-stats = Stats in the game
save-value = Value
save-iv = IV
save-ev = EV
save-moves = Moves in the game
save-pp = PP { $pp }/{ $max }
save-pp-left = PP { $pp }
save-unknown-move = Unknown move #{ $id }
save-randomizer-note = Stats, ability and moves come from the save. With a randomizer they may not match the Pokédex; types and sprites come from the Pokédex.
nature-hardy = Hardy
nature-lonely = Lonely
nature-brave = Brave
nature-adamant = Adamant
nature-naughty = Naughty
nature-bold = Bold
nature-docile = Docile
nature-relaxed = Relaxed
nature-impish = Impish
nature-lax = Lax
nature-timid = Timid
nature-hasty = Hasty
nature-serious = Serious
nature-jolly = Jolly
nature-naive = Naive
nature-modest = Modest
nature-mild = Mild
nature-quiet = Quiet
nature-bashful = Bashful
nature-rash = Rash
nature-calm = Calm
nature-gentle = Gentle
nature-sassy = Sassy
nature-careful = Careful
nature-quirky = Quirky
team-coverage = Team defensive coverage
team-coverage-weak = weak
team-coverage-resist = resist or immune
team-coverage-empty = Add members to see the coverage.
team-coverage-ok = No serious weak spots.
team-coverage-holes = Weak spots: { $types } (2 or more members weak and none resists).
compare-title = Comparator
compare-search = Search Pokémon…
compare-pick = Pick a Pokémon
compare-best-type = Its best type against { $name }: { $multiplier }
compare-swap = Swap sides
vs = VS
versus-title = Battle
versus-attacks = { $attacker } attacks { $defender }
versus-stab = With its own types
versus-lean-physical = Better physical: Attack { $attack } against Defense { $defense }.
versus-lean-special = Better special: Sp. Atk { $attack } against Sp. Def { $defense }.
versus-best-moves = Best moves it can learn
versus-no-moves = None of its damaging moves affects { $defender }.
versus-threats = Super effective types of { $defender }
versus-threats-none = None of its types is super effective.
versus-hits = { $hits ->
    [one] 1 hit
   *[other] { $hits } hits
}
versus-hits-turns = { $hits ->
    [one] 1 hit
   *[other] { $hits } hits
} ({ $turns } turns)
versus-charge = charges first
versus-recharge = must recharge
versus-winner = { $name } has the edge: it knocks { $other } out in { $hits } with { $attack }.
versus-winner-faster = { $name } has the edge: both need { $hits }, but it moves first.
versus-loser = { $name } would need { $hits } with { $attack }.
versus-loser-none = { $name } can't damage { $other }.
versus-even = Even fight: both need { $hits } and have the same Speed.
versus-nobody = Neither can damage the other.
versus-faster = { $name } moves first (Speed { $speed } against { $other }).
versus-speed-tie = Same Speed ({ $speed }): who moves first is random.
versus-assumptions = Estimate at level { $level } with perfect IVs, no EVs, a neutral nature, no items, abilities or critical hits, using the move each one can learn in any game that knocks the other out in the fewest turns (charging and recharging take an extra turn).
all-types = All
select-pokemon = Select a Pokémon
select-pokemon-hint = Pick one from the list, browse with ↑ ↓ or press / to search.

## Main page: items

items-search-placeholder = Search items by name
items-all-pockets = All pockets
items-name = Name
items-range = { $start }–{ $end } of { $total } items
items-none = No items
items-no-results = No items found
items-no-results-hint = Try another search or pick another pocket.
item-details = Item
item-info = Information
item-pocket = Pocket
item-category = Category
item-cost = Price
item-cost-value = { $cost } ₽
item-cost-none = Not sold
item-description = Description
item-effect = Effect
item-held-by = Held by wild Pokémon

## Main page: type chart

type-chart = Type chart
type-chart-hint = Each row is the attacking type and each column the defending type. Pick a type on the left to see its matchups.
type-chart-select-hint = Pick a type in the chart below to see what it is strong and weak against.
type-chart-corner = Attack ↓ · Defense →
type-chart-legend-super = 2× super effective
type-chart-legend-not-very = ½× not very effective
type-chart-legend-none = 0× no effect
type-chart-legend-neutral = 1× normal damage
type-chart-none = None
type-matchups-title = Matchups
type-chart-hover = { $attacker } → { $defender }: { $multiplier }
type-chart-hover-hint = Hover a cell to see the matchup.
type-calculator = Defensive calculator
type-calculator-hint = Pick up to two types to see what hurts them.
type-calculator-clear = Clear
type-matchups = { $name } matchups
type-offense = Attacking
type-defense = Defending
super-effective-against = Deals 2× damage to
not-very-effective-against = Deals ½ damage to
no-effect-against = Has no effect on

## Main page: Pokémon list

# Toolbar: search and sorting
search-placeholder = Name or number…
sort-by = Sort by
sort-number = Number
sort-name = Name
reverse-order = Reverse order
order-ascending = ↑ Asc
order-descending = ↓ Desc
clear-all = Clear search and filters

# Empty results
no-results = No Pokémon found
no-results-hint = Try another search or remove some filters.

# Footer: result count and pagination
results-range = { $start }–{ $end } of { $total } Pokémon
results-none = No Pokémon
page-of = Page { $page } of { $pages }


## Filters drawer

filters-page = Filters
reset-filters = Reset
results-count = { $count ->
    [one] 1 Pokémon matches
   *[other] { $count } Pokémon match
}

# Sections
type-filters = Filter by Type
type-filter-mode-hint = The type filtering mode (match any/all selected types) can be changed in Settings.
generation-filters = Filter by Generation
category-filters = Filter by Category
stats-filter = Filter by Pokémon Stats
minimum-poke-stats = Minimum base stat total


## Pokémon details page: header and tabs

back = Back
back-to-list = Pokédex
previous-pokemon = Previous Pokémon
next-pokemon = Next Pokémon

# Tab bar
tab-overview = Overview
tab-stats = Stats
tab-moves = Moves
tab-evolution = Evolution
tab-locations = Locations
tab-forms = Forms

# Game picker (Moves and Locations tabs)
game = Game
all-games = All games


## Details › Overview tab

# Sprite card: view toggles and cries
shiny = Shiny
back-sprite = Back
female = Female
cry = Cry
cry-legacy = Classic cry

# Pokédex entry card
pokedex-entry = Pokédex entry
entry-from = From
no-flavor-text = No Pokédex entry available...

# Profile card
profile = Profile
height-label = Height
weight-label = Weight
habitat = Habitat
color = Color
shape = Shape
form = Form
alternate-form = Alternate form

# Abilities card
pokemon-abilities = Abilities
hidden-ability = Hidden

# Training card
training = Training
ev-yield = EV yield
base-exp = Base experience
catch-rate = Catch rate
base-friendship = Base friendship
growth-rate = Growth rate
exp-to-100 = { $exp } exp. to Lv. 100

# Breeding card
breeding = Breeding
egg-groups = Egg groups
gender = Gender
genderless = Genderless
egg-cycles = Egg cycles
egg-cycles-value = { $cycles } (~{ $steps } steps)

# Other cards
regional-numbers = Regional Pokédex numbers
held-items = Wild held items
appears-in = Appears in


## Details › Stats tab

# Base stats card
poke-stats = Base stats
stat = Stat
base = Base
min-100 = Min
max-100 = Max
total = Total
base-total = Base total
stat-range-hint = Min/Max: stat range at level 100 (0 IVs/0 EVs/hindering nature to 31 IVs/252 EVs/beneficial nature).

# Type defenses card
type-defenses = Type defenses
weak-2x = Takes 2× damage from
resist-half = Takes ½ damage from
immune = Immune to
very-weak = Very weak
weak = Weak
resists = Resists
very-resistant = Very resistant
immune-label = Immune

# Previous types card
past-types = Previous types
types-until = Until { $generation }


## Details › Moves tab

no-moves = No move data available...

# Table columns
learned = Learned
move = Move
category = Category
power-short = Power
accuracy-short = Acc.
pp = PP

# "Learned" column values
level-short = Lv. { $level }
evolve-short = Evolve

# Learn methods (table titles)
level-up = Level up
tm = TM/HM
egg = Egg
tutor = Tutor
form-change = Form change
other-method = Other

# Damage classes
physical = Physical
special = Special
status = Status

# Expanded move details
target = Target
priority = Priority
introduced = Introduced
high-crit = High critical-hit ratio
drain = Drains { $value }% of damage
recoil = { $value }% recoil
flinch = { $value }% flinch chance
hits = Hits { $min }–{ $max } times


## Details › Evolution tab

poke-evo-data = Evolution chain
no-evo-data = This Pokémon does not evolve.
baby-trigger = Breeding the baby requires a parent holding { $item }

# Evolution conditions. Several can be joined with " · ", e.g. "Level up · Friendship 160+ · Daytime"
evo-level = Level { $level }
evo-level-up = Level up
evo-trade = Trade
evo-trade-for = Trade for { $species }
evo-use-item = Use { $item }
evo-holding = Holding { $item }
evo-happiness = Friendship { $value }+
evo-affection = Affection { $value }+
evo-beauty = Beauty { $value }+
evo-day = Daytime
evo-night = Nighttime
evo-dusk = Dusk
evo-female = Female
evo-male = Male
evo-knowing = Knowing { $name }
evo-knowing-type = Knowing a { $name }-type move
evo-at = At { $place }
evo-party-species = With { $name } in the party
evo-party-type = With a { $name }-type Pokémon in the party
evo-atk-gt-def = Attack > Defense
evo-atk-lt-def = Attack < Defense
evo-atk-eq-def = Attack = Defense
evo-used-move = Use { $name } { $count } times
evo-steps = Walk { $count } steps
evo-damage = Take { $count } damage
evo-rain = While raining
evo-upside-down = Console upside down
evo-multiplayer = In Union Circle
evo-chance = { $value }% chance


## Details › Locations tab

no-encounter-info = This Pokémon can't be found in the wild. Evolve, breed or trade to get it.


## Details › Forms tab

# Forms card
forms = Forms
no-forms = This Pokémon has no other forms.

# Sprites card
sprites = Sprites
no-sprites = No sprites available.
sprite-front = Front
sprite-back = Back
sprite-front-shiny = Front (shiny)
sprite-back-shiny = Back (shiny)
sprite-front-female = Front (female)
sprite-back-female = Back (female)
sprite-front-shiny-female = Front (shiny, female)
sprite-back-shiny-female = Back (shiny, female)


## Shared Pokémon vocabulary
## Used across several pages (list, filters, details).

# Types
normal = Normal
fire = Fire
water = Water
electric = Electric
grass = Grass
ice = Ice
fighting = Fighting
poison = Poison
ground = Ground
flying = Flying
psychic = Psychic
bug = Bug
rock = Rock
ghost = Ghost
dragon = Dragon
dark = Dark
steel = Steel
fairy = Fairy

# Stats: full names
hp = HP
attack = Attack
defense = Defense
sp-a = Special Attack
sp-d = Special Defense
spd = Speed
accuracy = Accuracy
evasion = Evasion

# Stats: short names (charts, compact tables, EV yield)
atk-short = Atk
def-short = Def
spa-short = SpA
spd-short = SpD
spe-short = Spe

# Generations
gen-i = Gen I
gen-ii = Gen II
gen-iii = Gen III
gen-iv = Gen IV
gen-v = Gen V
gen-vi = Gen VI
gen-vii = Gen VII
gen-viii = Gen VIII
gen-ix = Gen IX
unknown = Unknown

# Categories (filters and profile tags)
category-regular = Regular
category-baby = Baby
category-legendary = Legendary
category-mythical = Mythical

# Growth rates
growth-slow = Slow
growth-medium = Medium Fast
growth-fast = Fast
growth-medium-slow = Medium Slow
growth-erratic = Erratic
growth-fluctuating = Fluctuating

# Egg groups
egg-monster = Monster
egg-water1 = Water 1
egg-water2 = Water 2
egg-water3 = Water 3
egg-bug = Bug
egg-flying = Flying
egg-field = Field
egg-fairy = Fairy
egg-grass = Grass
egg-human-like = Human-Like
egg-mineral = Mineral
egg-amorphous = Amorphous
egg-ditto = Ditto
egg-dragon = Dragon
egg-undiscovered = Undiscovered
