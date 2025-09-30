use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Axial { pub q: i32, pub r: i32 } // axial hex coords

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Biome { Forest, Hill, Pasture, Mountain, Field, Wastes } // your names

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Resource { Timber, Brick, Fiber, Ore, Grain }

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Build { Road, Outpost, Town }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub at: Axial,
    pub biome: Biome,
    pub token: u8,           // 2..12 or 0 for no production
    pub blocked: bool,       // event effects can block
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Inventory(pub BTreeMap<Resource, u8>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub inventory: Inventory,
    pub routes: BTreeSet<(Axial, Axial)>,
    pub outposts: BTreeSet<Axial>,
    pub towns: BTreeSet<Axial>,
    pub vp_tokens: u8,          // objective cards etc.
    pub knights_played: u8,     // renamed: “guards” if you add that later
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Port {
    pub on_edge: (Axial, Axial),
    pub rate: (u8, u8),         // e.g., (3,1) or (2,1)
    pub specific: Option<Resource>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    pub tiles: BTreeMap<Axial, Tile>,
    pub ports: Vec<Port>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Costs {
    pub road: BTreeMap<Resource, u8>,
    pub outpost: BTreeMap<Resource, u8>,
    pub town: BTreeMap<Resource, u8>,
    pub dev_card: BTreeMap<Resource, u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rules {
    pub win_vp: u8,                    // default 10
    pub general_trade_rate: (u8, u8),  // default (4,1)
    pub costs: Costs,
    pub distance_rule: u8,             // edges between settlements: default 2
    pub route_bonus_len: u8,           // route needed to claim bonus
    pub route_bonus_vp: u8,            // default 2
    pub army_bonus_req: u8,            // if you use “army” style cards
    pub army_bonus_vp: u8,             // default 2
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventCard {
    RichHarvest { biome: Biome, bonus: u8 },
    Drought { biome: Biome },
    TradeEmbargo, // only bank trades this round
    OpenSeas,     // port rates improve (3:1 -> 2:1)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub map: Map,
    pub rules: Rules,
    pub event_deck: Vec<EventCard>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnLogEntry {
    pub player: usize,
    pub action: String,
}