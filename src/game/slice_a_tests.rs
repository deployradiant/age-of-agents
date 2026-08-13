use super::*;

#[test]
fn roadmap_catalog_has_exact_stable_wire_contract() {
    let catalog_json = serde_json::to_value(DomainCatalog::roadmap()).unwrap();
    assert_eq!(
        catalog_json,
        serde_json::json!({
            "resources": [
                "wood", "food", "stone", "gold", "iron", "coal", "clay", "fiber",
                "timber", "steel", "bricks", "cloth", "rations"
            ],
            "buildings": [
                "town_center", "mining_camp", "farm", "lumber_mill", "smelter", "kiln",
                "weaver", "kitchen", "barracks", "range", "workshop", "infirmary",
                "watchtower", "monument"
            ],
            "units": ["villager", "guard", "archer", "healer", "siege_cart"],
            "recipes": [
                {"building": "lumber_mill", "inputs": ["wood"], "output": "timber"},
                {"building": "smelter", "inputs": ["iron", "coal"], "output": "steel"},
                {"building": "kiln", "inputs": ["clay", "wood"], "output": "bricks"},
                {"building": "weaver", "inputs": ["fiber"], "output": "cloth"},
                {"building": "kitchen", "inputs": ["food"], "output": "rations"}
            ],
            "technologies": ["forestry", "agriculture", "masonry", "mining", "textiles"]
        })
    );
    assert_eq!(
        GameWorld::default().snapshot().catalog,
        DomainCatalog::roadmap()
    );
}

#[test]
fn scenario_reaches_loss_on_the_exact_authoritative_tick_boundary() {
    let mut before = GameWorld::default();
    before.scenario.elapsed_ticks = 7;
    before.scenario.tick_limit = 9;
    let mut repeat = before.clone();

    before.tick(0.1);
    repeat.tick(0.1);
    assert_eq!(before, repeat);
    assert_eq!(before.scenario.elapsed_ticks, 8);
    assert_eq!(before.scenario.outcome, ScenarioOutcome::Running);

    before.tick(0.1);
    assert_eq!(before.scenario.elapsed_ticks, 9);
    assert_eq!(before.scenario.outcome, ScenarioOutcome::Lost);
    assert_eq!(before.snapshot().scenario, before.scenario);
}

#[test]
fn terminal_scenario_progress_freezes_while_gameplay_remains_deferred() {
    for outcome in [ScenarioOutcome::Won, ScenarioOutcome::Lost] {
        let mut world = GameWorld::default();
        world.scenario.elapsed_ticks = 12;
        world.scenario.outcome = outcome;
        let game_tick = world.tick;

        world.tick(0.1);

        assert_eq!(world.scenario.elapsed_ticks, 12);
        assert_eq!(world.scenario.outcome, outcome);
        assert_eq!(world.tick, game_tick + 1);
    }
}

#[test]
fn paused_and_invalid_ticks_do_not_advance_the_scenario_limit() {
    let mut world = GameWorld::default();
    world.scenario.elapsed_ticks = world.scenario.tick_limit - 1;
    world.simulation_speed = 0.0;

    world.tick(1.0);
    world.tick(f64::NAN);
    world.tick(-1.0);
    assert_eq!(world.scenario.elapsed_ticks, world.scenario.tick_limit - 1);
    assert_eq!(world.scenario.outcome, ScenarioOutcome::Running);
}
