// ...existing code...
#[test]
fn mvp_loop() {
    use std::collections::{BTreeMap, BTreeSet};
    use hexlands_core::engine::Game;
    use hexlands_core::types::*;

    // Build simple rules/costs
    let mut road = BTreeMap::new();
    road.insert(Resource::Timber, 1);
    road.insert(Resource::Brick, 1);

    let mut outpost = BTreeMap::new();
    outpost.insert(Resource::Timber, 1);
    outpost.insert(Resource::Brick, 1);
    outpost.insert(Resource::Fiber, 1);
    outpost.insert(Resource::Grain, 1);

    let mut town = BTreeMap::new();
    town.insert(Resource::Grain, 2);
    town.insert(Resource::Ore, 3);

    let mut dev = BTreeMap::new();
    dev.insert(Resource::Grain, 1);
    dev.insert(Resource::Fiber, 1);
    dev.insert(Resource::Ore, 1);

    let costs = Costs {
        road,
        outpost,
        town,
        dev_card: dev,
    };

    let rules = Rules {
        win_vp: 10,
        general_trade_rate: (4, 1),
        costs,
        distance_rule: 2,
        route_bonus_len: 5,
        route_bonus_vp: 2,
        army_bonus_req: 3,
        army_bonus_vp: 2,
    };

    // Single tile that produces on roll 6
    let mut tiles = BTreeMap::new();
    let a0 = Axial { q: 0, r: 0 };
    tiles.insert(
        a0,
        Tile {
            at: a0,
            biome: Biome::Forest,
            token: 6,
            blocked: false,
        },
    );

    let map = Map { tiles, ports: vec![] };

    let cfg = Config {
        map,
        rules,
        event_deck: vec![],
    };

    // Create 3 players; player 0 has inventory seeded and an outpost to receive production
    let mut p0_inv = BTreeMap::new();
    p0_inv.insert(Resource::Timber, 2);
    p0_inv.insert(Resource::Brick, 1);
    p0_inv.insert(Resource::Fiber, 4); // enough for a bank trade (we'll use rate (1,1) in test)
    p0_inv.insert(Resource::Grain, 1);
    p0_inv.insert(Resource::Ore, 1);

    let p0 = Player {
        id: 0,
        name: "Alice".into(),
        inventory: Inventory(p0_inv),
        routes: BTreeSet::new(),
        outposts: {
            let mut s = BTreeSet::new();
            s.insert(a0); // adjacent (stubs accept)
            s
        },
        towns: BTreeSet::new(),
        vp_tokens: 0,
        knights_played: 0,
    };

    let p1 = Player {
        id: 1,
        name: "Bob".into(),
        inventory: Inventory(BTreeMap::new()),
        routes: BTreeSet::new(),
        outposts: BTreeSet::new(),
        towns: BTreeSet::new(),
        vp_tokens: 0,
        knights_played: 0,
    };

    let p2 = Player {
        id: 2,
        name: "Carol".into(),
        inventory: Inventory(BTreeMap::new()),
        routes: BTreeSet::new(),
        outposts: BTreeSet::new(),
        towns: BTreeSet::new(),
        vp_tokens: 0,
        knights_played: 0,
    };

    // seed bank with plenty of timber and grain
    let mut bank_map = BTreeMap::new();
    bank_map.insert(Resource::Timber, 50u8);
    bank_map.insert(Resource::Grain, 50u8);
    bank_map.insert(Resource::Fiber, 50u8);
    bank_map.insert(Resource::Ore, 50u8);
    bank_map.insert(Resource::Brick, 50u8);

    let mut game = Game {
        cfg,
        players: vec![p0, p1, p2],
        bank: Inventory(bank_map),
        route_bonus_owner: None,
        army_bonus_owner: None,
        discard_mode: false,
        log: vec![],
    };

    // 1) Produce on roll 6 -> player 0 should receive at least one Timber (adjacent outpost)
    game.produce(6);
    let alice_timber = game.players[0].inventory.0.get(&Resource::Timber).copied().unwrap_or(0);
    assert!(alice_timber >= 3, "alice should have been paid by produce (initial 2 + at least 1 payout)");

    // 2) Build an outpost somewhere (spend uses configured costs); choose an axial
    let new_v = Axial { q: 1, r: 0 };
    // give Alice one more Brick to satisfy outpost cost if necessary
    *game.players[0].inventory.0.entry(Resource::Brick).or_default() += 1;
    game.build(0, Build::Outpost, Some(new_v), None).expect("build outpost failed");
    assert!(game.players[0].outposts.contains(&new_v), "outpost must be present after build");

    // 3) Trade with bank: use a trivial (1,1) rate to exercise the logic
    let rate = (1u8, 1u8);
    game.trade_with_bank(0, Resource::Fiber, Resource::Grain, rate).expect("trade with bank failed");
    assert!(game.players[0].inventory.0.get(&Resource::Grain).copied().unwrap_or(0) >= 1);

    // 4) Give route bonus to player 0 and validate vp math
    game.route_bonus_owner = Some(0);
    // force vp_tokens to a known value and ensure vp includes builds + tokens + route bonus
    game.players[0].vp_tokens = 1;
    let computed = game.vp(&game.players[0]);
    // builds: each outpost = 1vp; player has at least two outposts now -> >=2, plus vp_tokens 1 plus route_bonus_vp 2
    assert!(computed >= 5, "vp math should add builds, tokens and route bonus (got {})", computed);
}
// ...existing code...