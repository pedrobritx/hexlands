use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub struct Game {
    pub cfg: Config,
    pub players: Vec<Player>,
    pub bank: Inventory,
    pub route_bonus_owner: Option<usize>,
    pub army_bonus_owner: Option<usize>,
    pub discard_mode: bool, // set true when 7-like effects happen (optional)
    pub log: Vec<TurnLogEntry>,
}

impl Game {
    pub fn vp(&self, p: &Player) -> u8 {
        let builds = (p.outposts.len() as u8) * 1 + (p.towns.len() as u8) * 2;
        let route = if Some(p.id) == self.route_bonus_owner { self.cfg.rules.route_bonus_vp } else { 0 };
        let army  = if Some(p.id) == self.army_bonus_owner  { self.cfg.rules.army_bonus_vp }  else { 0 };
        builds + p.vp_tokens + route + army
    }

    pub fn produce(&mut self, roll: u8) {
        // grant resources to adjacent outposts/towns for tiles matching token
        let mut payout: Vec<(usize, Resource)> = vec![];
        for (ax, tile) in &self.cfg.map.tiles {
            if tile.token != roll || tile.blocked { continue; }
            let res = biome_to_res(tile.biome);
            for (pid, p) in self.players.iter().enumerate() {
                let mut amt = 0u8;
                if p.outposts.iter().any(|v| adjacent(*v, *ax)) { amt += 1; }
                if p.towns.iter().any(|v| adjacent(*v, *ax))   { amt += 2; }
                for _ in 0..amt { payout.push((pid, res)); }
            }
        }
        // bank pays out if possible; if multiple affected and bank runs dry, nobody gets that type.
        let mut need_by_res: BTreeMap<Resource, usize> = BTreeMap::new();
        for &(_, r) in &payout { *need_by_res.entry(r).or_default() += 1; }
        for (r, need) in need_by_res {
            let have = *self.bank.0.get(&r).unwrap_or(&0) as usize;
            if have >= need {
                // fulfill
                for (pid, rr) in payout.iter().cloned().filter(|(_, rr)| *rr == r) {
                    *self.players[pid].inventory.0.entry(r).or_default() += 1;
                    *self.bank.0.entry(r).or_default() -= 1;
                }
            }
            // else: if more than one player would receive, nobody gets it; if exactly one, give remainder.
            // keep it simple for MVP: deny all if insufficient.
        }
        self.log.push(TurnLogEntry{ player: 999, action: format!("produce {roll}") });
    }

    pub fn can_build(&self, pid: usize, b: Build, at: Option<Axial>) -> Result<(), &'static str> {
        let p = &self.players[pid];
        match b {
            Build::Road => {
                let (a,b) = at_edge(at)?;
                // must connect to existing network and not pass through enemy settlement
                if !connected_to_player(p, a, b) { return Err("not connected"); }
                if edge_blocked_by_enemy_settlement(&self.players, a, b) { return Err("blocked"); }
                Ok(())
            }
            Build::Outpost => {
                let v = at.ok_or("need vertex")?;
                if !distance_rule_ok(&self.players, v, self.cfg.rules.distance_rule) { return Err("distance rule"); }
                if !touches_player_network(p, v) { return Err("must touch your road"); }
                Ok(())
            }
            Build::Town => {
                let v = at.ok_or("need vertex")?;
                if !p.outposts.contains(&v) { return Err("upgrade requires outpost"); }
                Ok(())
            }
        }
    }

    pub fn spend(&mut self, pid: usize, cost: &BTreeMap<Resource,u8>) -> Result<(), &'static str> {
        for (r, &amt) in cost {
            if self.players[pid].inventory.0.get(r).copied().unwrap_or(0) < amt { return Err("insufficient"); }
        }
        for (r, &amt) in cost {
            *self.players[pid].inventory.0.entry(*r).or_default() -= amt;
            *self.bank.0.entry(*r).or_default() += amt;
        }
        Ok(())
    }

    pub fn build(&mut self, pid: usize, b: Build, at: Option<Axial>, edge: Option<(Axial,Axial)>) -> Result<(), &'static str> {
        let cost = match b {
            Build::Road => &self.cfg.rules.costs.road,
            Build::Outpost => &self.cfg.rules.costs.outpost,
            Build::Town => &self.cfg.rules.costs.town,
        }.clone();
        self.can_build(pid, b, at)?;
        self.spend(pid, &cost)?;
        match b {
            Build::Road => {
                let (a,b) = edge.ok_or("edge needed")?;
                self.players[pid].routes.insert((a,b));
                self.recompute_route_bonus();
            }
            Build::Outpost => { self.players[pid].outposts.insert(at.unwrap()); }
            Build::Town => {
                let v = at.unwrap();
                self.players[pid].outposts.remove(&v);
                self.players[pid].towns.insert(v);
            }
        }
        Ok(())
    }

    pub fn trade_with_bank(&mut self, pid: usize, give: Resource, get: Resource, rate: (u8,u8)) -> Result<(), &'static str> {
        let (g, _) = rate;
        let inv = &mut self.players[pid].inventory.0;
        if inv.get(&give).copied().unwrap_or(0) < g { return Err("insufficient"); }
        *inv.entry(give).or_default() -= g;
        *self.bank.0.entry(give).or_default() += g;
        if *self.bank.0.get(&get).unwrap_or(&0) == 0 { return Err("bank empty"); }
        *self.bank.0.entry(get).or_default() -= 1;
        *inv.entry(get).or_default() += 1;
        Ok(())
    }

    fn recompute_route_bonus(&mut self) {
        let req = self.cfg.rules.route_bonus_len as usize;
        let mut best: Option<(usize, usize)> = None; // (pid, length)
        for p in &self.players {
            let l = longest_path(&p.routes);
            if l >= req {
                if best.map(|(_,bl)| l > bl).unwrap_or(true) { best = Some((p.id, l)); }
            }
        }
        self.route_bonus_owner = best.map(|(pid,_)| pid);
    }
}

// helpers (stubs for brevity)
fn biome_to_res(b: Biome) -> Resource {
    match b { Biome::Forest => Resource::Timber, Biome::Hill => Resource::Brick, Biome::Pasture => Resource::Fiber, Biome::Mountain => Resource::Ore, Biome::Field => Resource::Grain, Biome::Wastes => Resource::Grain /* or none */ }
}
fn adjacent(_a: Axial, _b: Axial) -> bool { /* axial math */ true }
fn at_edge(at: Option<Axial>) -> Result<(Axial,Axial), &'static str> { Err("edge coord pair required") }
fn connected_to_player(_p: &Player, _a: Axial, _b: Axial) -> bool { true }
fn edge_blocked_by_enemy_settlement(_players: &Vec<Player>, _a: Axial, _b: Axial) -> bool { false }
fn distance_rule_ok(_players: &Vec<Player>, _v: Axial, _min_edges: u8) -> bool { true }
fn touches_player_network(_p: &Player, _v: Axial) -> bool { true }
fn longest_path(_edges: &BTreeSet<(Axial,Axial)>) -> usize { 0 }