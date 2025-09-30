use crate::types::*;

pub fn apply_event(cfg: &Rules, ev: &EventCard, map: &mut Map) {
    match ev {
        EventCard::RichHarvest{ biome, bonus: _ } => {
            for tile in map.tiles.values_mut() {
                if &tile.biome == biome { /* mark for bonus this round */ }
            }
        }
        EventCard::Drought{ biome } => {
            for tile in map.tiles.values_mut() {
                if &tile.biome == biome { tile.blocked = true; }
            }
        }
        EventCard::TradeEmbargo => { /* set a flag in game to disallow player-to-player trades */ }
        EventCard::OpenSeas => { /* temporarily improve port rates */ }
    }
}