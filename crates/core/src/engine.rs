use crate::types::*;
use std::collections::{BTreeMap, BTreeSet};
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

    pub fn can_build(&self, pid: usize, b: Build, at: Option<Axial>, edge: Option<(Axial,Axial)>) -> Result<(), &'static str> {
        let p = &self.players[pid];
        match b {
            Build::Road => {
                let (a,b) = at_edge(edge)?;
                // must connect to existing network and not pass through enemy settlement
                if !connected_to_player(p, a, b) { return Err("not connected"); }
                if edge_blocked_by_enemy_settlement(&self.players, pid, a, b) { return Err("blocked"); }
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
        self.can_build(pid, b, at, edge)?;
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

// helpers (implemented)
fn biome_to_res(b: Biome) -> Resource {
    match b {
        Biome::Forest => Resource::Timber,
        Biome::Hill => Resource::Brick,
        Biome::Pasture => Resource::Fiber,
        Biome::Mountain => Resource::Ore,
        Biome::Field => Resource::Grain,
        Biome::Wastes => Resource::Grain,
    }
}

fn adjacent(a: Axial, b: Axial) -> bool {
    let dq = b.q - a.q;
    let dr = b.r - a.r;
    matches!((dq, dr), (1,0) | (0,1) | (-1,0) | (0,-1) | (1,-1) | (-1,1))
}

fn at_edge(e: Option<(Axial,Axial)>) -> Result<(Axial,Axial), &'static str> {
    e.ok_or("edge coord pair required")
}

fn connected_to_player(p: &Player, a: Axial, b: Axial) -> bool {
    // simple rule: the player has a route that touches either endpoint OR a settlement at either endpoint
    if p.routes.iter().any(|(x,y)| *x == a || *y == a || *x == b || *y == b) { return true; }
    if p.outposts.contains(&a) || p.outposts.contains(&b) || p.towns.contains(&a) || p.towns.contains(&b) { return true; }
    false
}

fn edge_blocked_by_enemy_settlement(players: &Vec<Player>, owner_pid: usize, a: Axial, b: Axial) -> bool {
    // If any other player's outpost/town is located on either endpoint, they block building through that vertex
    for p in players.iter() {
        if p.id == owner_pid { continue; }
        if p.outposts.contains(&a) || p.outposts.contains(&b) || p.towns.contains(&a) || p.towns.contains(&b) { return true; }
    }
    false
}

fn distance_rule_ok(players: &Vec<Player>, v: Axial, _min_edges: u8) -> bool {
    // For now: ensure no other player's outpost/town is adjacent (distance rule = number of edges between settlements)
    for p in players.iter() {
        for &ov in p.outposts.iter().chain(p.towns.iter()) {
            if adjacent(ov, v) { return false; }
        }
    }
    true
}

fn touches_player_network(p: &Player, v: Axial) -> bool {
    // A vertex touches the player's network if any route endpoint equals v
    p.routes.iter().any(|(x,y)| *x == v || *y == v)
}

fn longest_path(edges: &BTreeSet<(Axial,Axial)>) -> usize {
    use std::collections::BTreeMap;
    let mut adj: BTreeMap<Axial, Vec<Axial>> = BTreeMap::new();
    for &(a,b) in edges.iter() {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    fn dfs(node: Axial, adj: &BTreeMap<Axial, Vec<Axial>>, visited: &mut BTreeSet<Axial>) -> usize {
        let mut best = 0usize;
        if let Some(neis) = adj.get(&node) {
            for &n in neis {
                if visited.contains(&n) { continue; }
                visited.insert(n);
                let len = 1 + dfs(n, adj, visited);
                visited.remove(&n);
                if len > best { best = len; }
            }
        }
        best
    }

    let mut best_overall = 0usize;
    for &start in adj.keys() {
        let mut visited = BTreeSet::new();
        visited.insert(start);
        let l = dfs(start, &adj, &mut visited);
        if l > best_overall { best_overall = l; }
    }
    best_overall
}