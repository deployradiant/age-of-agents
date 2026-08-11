use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::*;

#[test]
fn voronoi_terrain_is_fixed_and_deterministic() {
    let first = GameWorld::default();
    let second = GameWorld::default();
    assert_eq!(first.width, 2400.0);
    assert_eq!(first.height, 1600.0);
    assert_eq!(first.cell_size, 80.0);
    assert_eq!(first.terrain, second.terrain);
}

#[test]
fn resources_are_deterministic_separated_bounded_and_biome_compatible() {
    let world = GameWorld::default();
    assert_eq!(world.resources, GameWorld::default().resources);

    let counts = world
        .resources
        .iter()
        .fold(BTreeMap::new(), |mut counts, resource| {
            *counts.entry(resource.kind).or_insert(0) += 1;
            counts
        });
    assert_eq!(
        counts,
        BTreeMap::from([
            (ResourceKind::Wood, 6),
            (ResourceKind::Food, 4),
            (ResourceKind::Stone, 4),
            (ResourceKind::Gold, 2),
            (ResourceKind::Iron, 2),
            (ResourceKind::Clay, 2),
            (ResourceKind::Fiber, 2),
        ])
    );

    for (index, resource) in world.resources.iter().enumerate() {
        assert!((0.0..=world.width).contains(&resource.position.x));
        assert!((0.0..=world.height).contains(&resource.position.y));
        assert!(
            resource.position.distance(Position {
                x: 1200.0,
                y: 800.0
            }) >= STARTING_BASE_RESOURCE_CLEARANCE
        );
        let coordinate = world.cell_for_position(resource.position).unwrap();
        let biome = world
            .terrain
            .iter()
            .find(|cell| cell.coordinate() == coordinate)
            .unwrap()
            .biome;
        assert!(compatible_biomes(resource.kind).contains(&biome));
        for other in &world.resources[index + 1..] {
            assert!(
                resource.position.distance(other.position) + f64::EPSILON
                    >= RESOURCE_MIN_SEPARATION,
                "{} overlaps {}",
                resource.id,
                other.id
            );
        }
    }
}

#[test]
fn voronoi_terrain_contains_exactly_all_eight_biomes() {
    let biomes: BTreeSet<_> = GameWorld::default()
        .terrain
        .iter()
        .map(|cell| cell.biome)
        .collect();
    assert_eq!(biomes.len(), 8);
}

#[test]
fn terrain_cells_are_unique_and_cover_the_world_bounds() {
    let world = GameWorld::default();
    let coordinates: BTreeSet<_> = world.terrain.iter().map(|cell| cell.coordinate()).collect();
    assert_eq!(world.terrain.len(), 600);
    assert_eq!(coordinates.len(), 600);
    assert!(
        coordinates
            .iter()
            .all(|cell| cell.column < WORLD_COLUMNS && cell.row < WORLD_ROWS)
    );
    assert!(coordinates.contains(&CellCoordinate { column: 0, row: 0 }));
    assert!(coordinates.contains(&CellCoordinate {
        column: WORLD_COLUMNS - 1,
        row: WORLD_ROWS - 1,
    }));
}

#[test]
fn every_voronoi_biome_is_one_coherent_region() {
    let world = GameWorld::default();
    let by_biome: BTreeMap<_, BTreeSet<_>> =
        world
            .terrain
            .iter()
            .fold(BTreeMap::new(), |mut regions, cell| {
                regions
                    .entry(cell.biome)
                    .or_default()
                    .insert(cell.coordinate());
                regions
            });

    for (biome, region) in by_biome {
        let mut reached = BTreeSet::new();
        let mut queue = VecDeque::from([*region.first().expect("biome is present")]);
        while let Some(cell) = queue.pop_front() {
            if !reached.insert(cell) {
                continue;
            }
            for (dx, dy) in [(1_i32, 0_i32), (-1, 0), (0, 1), (0, -1)] {
                let column = i32::from(cell.column) + dx;
                let row = i32::from(cell.row) + dy;
                if column >= 0 && row >= 0 {
                    let neighbor = CellCoordinate {
                        column: column as u16,
                        row: row as u16,
                    };
                    if region.contains(&neighbor) && !reached.contains(&neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        assert_eq!(reached, region, "{biome:?} is not coherent");
    }
}

#[test]
fn explored_cells_only_grow_as_units_move() {
    let mut world = GameWorld::default();
    let initially_explored: BTreeSet<_> = world.explored_cells.iter().copied().collect();
    world.units[0].position = Position { x: 80.0, y: 80.0 };
    world.units[1].position = Position { x: 80.0, y: 80.0 };
    world.tick(0.1);
    let after_move: BTreeSet<_> = world.explored_cells.iter().copied().collect();
    assert!(after_move.is_superset(&initially_explored));
    assert!(after_move.len() > initially_explored.len());

    world.units[0].position = Position {
        x: 2320.0,
        y: 1520.0,
    };
    world.units[1].position = Position {
        x: 2320.0,
        y: 1520.0,
    };
    world.tick(0.1);
    let after_second_move: BTreeSet<_> = world.explored_cells.iter().copied().collect();
    assert!(after_second_move.is_superset(&after_move));
    assert!(after_second_move.len() > after_move.len());
}

#[test]
fn move_command_walks_to_the_destination_and_reveals_the_route() {
    let mut world = GameWorld::default();
    let initially_explored = world.explored_cells.len();
    let destination = Position { x: 80.0, y: 80.0 };

    world
        .apply_command(Command::Move {
            unit_id: "villager-1".into(),
            x: destination.x,
            y: destination.y,
        })
        .unwrap();
    assert!(matches!(world.units[0].action, UnitAction::Move { .. }));

    world.tick(30.0);

    assert_eq!(world.units[0].position, destination);
    assert_eq!(world.units[0].action, UnitAction::Idle);
    assert!(world.explored_cells.len() > initially_explored);
}

#[test]
fn move_command_rejects_non_finite_and_out_of_bounds_destinations() {
    for (x, y) in [(-1.0, 80.0), (80.0, 1601.0), (f64::NAN, 80.0)] {
        let mut world = GameWorld::default();
        assert_eq!(
            world.apply_command(Command::Move {
                unit_id: "villager-1".into(),
                x,
                y,
            }),
            Err(CommandError::InvalidDestination)
        );
        assert_eq!(world.units[0].action, UnitAction::Idle);
    }
}

#[test]
fn snapshots_have_three_typed_fog_states_and_hide_never_seen_resources() {
    let mut world = GameWorld::default();
    world.units[0].position = Position { x: 80.0, y: 80.0 };
    world.units[1].position = Position { x: 80.0, y: 80.0 };
    world.tick(0.1);
    let snapshot = world.snapshot();
    let states: BTreeSet<_> = snapshot
        .terrain
        .iter()
        .map(|cell| cell.visibility)
        .collect();
    assert_eq!(
        states,
        BTreeSet::from([
            CellVisibility::Unseen,
            CellVisibility::Explored,
            CellVisibility::Visible,
        ])
    );
    assert!(
        snapshot
            .terrain
            .iter()
            .all(|cell| { cell.biome.is_none() == (cell.visibility == CellVisibility::Unseen) })
    );
    assert!(!snapshot.resources.is_empty());
    assert!(snapshot.resources.len() < world.resources.len());
    assert_eq!(snapshot.units.len(), 2);
}

#[test]
fn discovered_static_resources_do_not_pop_out_when_vision_moves_away() {
    let mut world = GameWorld::default();
    let discovered_resource_ids: BTreeSet<_> = world
        .snapshot()
        .resources
        .into_iter()
        .map(|resource| resource.id)
        .collect();
    assert!(!discovered_resource_ids.is_empty());

    world.units[0].position = Position { x: 80.0, y: 80.0 };
    world.units[1].position = Position { x: 80.0, y: 80.0 };
    world.tick(0.1);
    let remembered_resource_ids: BTreeSet<_> = world
        .snapshot()
        .resources
        .into_iter()
        .map(|resource| resource.id)
        .collect();

    assert!(remembered_resource_ids.is_superset(&discovered_resource_ids));
}

#[test]
fn completed_buildings_reveal_with_the_larger_building_radius() {
    let mut world = GameWorld::default();
    world.units.clear();
    world
        .buildings
        .push(town_center("building-sight", 1200.0, 800.0));
    world.explored_cells.clear();
    world.refresh_exploration();
    let building_visible = world
        .snapshot()
        .terrain
        .into_iter()
        .filter(|cell| cell.visibility == CellVisibility::Visible)
        .count();

    world.buildings.clear();
    world.units.push(Unit {
        id: "unit-sight".into(),
        position: Position {
            x: 1200.0,
            y: 800.0,
        },
        action: UnitAction::Idle,
        cargo: None,
    });
    let unit_visible = world
        .snapshot()
        .terrain
        .into_iter()
        .filter(|cell| cell.visibility == CellVisibility::Visible)
        .count();
    assert!(building_visible > unit_visible);
}

#[test]
fn idle_world_is_invariant_except_tick() {
    let mut world = GameWorld::default();
    let initial = world.clone();
    world.tick(10.0);
    assert_eq!(world.tick, 1);
    world.tick = 0;
    assert_eq!(world, initial);
}

fn start_gather_at_resource(world: &mut GameWorld, resource_index: usize, amount: f64) {
    let resource_id = world.resources[resource_index].id.clone();
    world.resources[resource_index].amount = amount;
    world.units[0].position = world.resources[resource_index].position;
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id,
        })
        .unwrap();
}

fn assert_gather_phase(unit: &Unit, expected: GatherPhase) {
    assert!(matches!(
        unit.action,
        UnitAction::Gather { phase, .. } if phase == expected
    ));
}

#[test]
fn partial_last_load_is_carried_then_deposited_before_becoming_idle() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 15.0);

    world.tick(15.0 / GATHER_RATE);
    assert_eq!(world.stockpile.wood, 0.0);
    assert_eq!(
        world.units[0].cargo,
        Some(CarriedResource {
            kind: ResourceKind::Wood,
            amount: 15.0,
        })
    );
    assert_eq!(world.resources[0].amount, 0.0);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);

    world.tick(100.0);
    assert_gather_phase(&world.units[0], GatherPhase::Depositing);
    assert_eq!(world.stockpile.wood, 0.0);
    world.tick(0.1);
    assert_eq!(world.stockpile.wood, 15.0);
    assert_eq!(world.units[0].cargo, None);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn full_load_deposits_and_resumes_the_same_gather_order() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 25.0);

    world.tick(VILLAGER_CARRY_CAPACITY / GATHER_RATE);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 20.0);
    assert_eq!(world.resources[0].amount, 5.0);
    assert_eq!(world.stockpile.wood, 0.0);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);
    world.tick(100.0);
    world.tick(0.1);

    assert_eq!(world.stockpile.wood, 20.0);
    assert_eq!(world.units[0].cargo, None);
    assert!(matches!(
        &world.units[0].action,
        UnitAction::Gather { resource_id, phase: GatherPhase::ToResource }
            if resource_id == "tree-1"
    ));
}

#[test]
fn repeated_round_trips_deposit_every_load_and_finish_after_depletion() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 45.0);

    for expected_stockpile in [20.0, 40.0, 45.0] {
        world.tick(100.0);
        assert_gather_phase(&world.units[0], GatherPhase::Returning);
        world.tick(100.0);
        assert_gather_phase(&world.units[0], GatherPhase::Depositing);
        world.tick(0.1);
        assert_eq!(world.stockpile.wood, expected_stockpile);
    }

    assert_eq!(world.resources[0].amount, 0.0);
    assert_eq!(world.units[0].cargo, None);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn depositing_is_exactly_once_across_later_ticks() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 10.0);
    world.tick(10.0 / GATHER_RATE);
    world.tick(100.0);
    world.tick(0.1);
    assert_eq!(world.stockpile.wood, 10.0);

    world.tick(100.0);
    assert_eq!(world.stockpile.wood, 10.0);
}

#[test]
fn deleted_resource_returns_existing_cargo_and_then_finishes() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 20.0);
    world.tick(0.5);
    assert_eq!(
        world.units[0].cargo.as_ref().unwrap().amount,
        GATHER_RATE * 0.5
    );
    world.resources.remove(0);

    world.tick(0.1);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);
    world.tick(100.0);
    world.tick(0.1);
    assert_eq!(world.stockpile.wood, GATHER_RATE * 0.5);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn missing_town_center_retains_cargo_until_a_deposit_is_possible() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 10.0);
    world.tick(10.0 / GATHER_RATE);
    let position = world.units[0].position;
    let town_center = world.buildings.remove(0);

    world.tick(100.0);
    assert_eq!(world.units[0].position, position);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 10.0);
    assert_eq!(world.stockpile.wood, 0.0);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);

    world.buildings.push(town_center);
    world.tick(100.0);
    world.tick(0.1);
    assert_eq!(world.stockpile.wood, 10.0);
    assert_eq!(world.units[0].action, UnitAction::Idle);
}

#[test]
fn default_world_has_seven_resource_kinds_and_a_productive_research_base() {
    let world = GameWorld::default();
    let kinds: BTreeSet<_> = world
        .resources
        .iter()
        .map(|resource| resource.kind)
        .collect();
    assert_eq!(
        kinds,
        BTreeSet::from([
            ResourceKind::Wood,
            ResourceKind::Food,
            ResourceKind::Stone,
            ResourceKind::Gold,
            ResourceKind::Iron,
            ResourceKind::Clay,
            ResourceKind::Fiber,
        ])
    );
    assert_eq!(world.buildings.len(), 1);
    assert_eq!(world.buildings[0].kind, BuildingKind::TownCenter);
    assert_eq!(world.buildings[0].produces, vec![ProductKind::Villager]);
    assert_eq!(world.buildings[0].researches, TechnologyKind::ALL);
    assert_eq!(world.buildings[0].job, None);
    assert!(world.researched_technologies.is_empty());
}

#[test]
fn depositing_routes_each_carried_resource_to_its_typed_stockpile() {
    for kind in [
        ResourceKind::Wood,
        ResourceKind::Food,
        ResourceKind::Stone,
        ResourceKind::Gold,
        ResourceKind::Iron,
        ResourceKind::Clay,
        ResourceKind::Fiber,
    ] {
        let mut world = GameWorld::default();
        let resource_index = world
            .resources
            .iter()
            .position(|resource| resource.kind == kind)
            .unwrap();
        start_gather_at_resource(&mut world, resource_index, 10.0);
        world.tick(10.0 / GATHER_RATE);
        assert_eq!(world.units[0].cargo.as_ref().unwrap().kind, kind);
        assert_eq!(
            world.stockpile.wood
                + world.stockpile.food
                + world.stockpile.stone
                + world.stockpile.gold
                + world.stockpile.iron
                + world.stockpile.clay
                + world.stockpile.fiber,
            0.0
        );

        world.tick(100.0);
        world.tick(0.1);
        let deposited = match kind {
            ResourceKind::Wood => world.stockpile.wood,
            ResourceKind::Food => world.stockpile.food,
            ResourceKind::Stone => world.stockpile.stone,
            ResourceKind::Gold => world.stockpile.gold,
            ResourceKind::Iron => world.stockpile.iron,
            ResourceKind::Clay => world.stockpile.clay,
            ResourceKind::Fiber => world.stockpile.fiber,
        };
        assert_eq!(deposited, 10.0);
        assert_eq!(world.resources[resource_index].amount, 0.0);
    }
}

#[test]
fn town_center_produces_one_villager_at_a_time_and_reserves_food_once() {
    let mut world = GameWorld::default();
    world.stockpile.food = 100.0;
    world
        .apply_command(Command::Produce {
            building_id: "base-1".into(),
            product: ProductKind::Villager,
        })
        .unwrap();
    assert_eq!(world.stockpile.food, 50.0);
    let before_rejected_command = world.clone();
    assert_eq!(
        world.apply_command(Command::Produce {
            building_id: "base-1".into(),
            product: ProductKind::Villager,
        }),
        Err(CommandError::BuildingBusy)
    );
    assert_eq!(world, before_rejected_command);

    world.tick(VILLAGER_PRODUCTION_SECONDS - 0.1);
    assert_eq!(world.units.len(), 2);
    assert!(world.buildings[0].job.is_some());
    world.tick(0.1);
    assert_eq!(world.units.len(), 3);
    assert_eq!(world.units[2].id, "villager-3");
    assert_eq!(world.units[2].action, UnitAction::Idle);
    assert_eq!(world.buildings[0].job, None);
    assert_eq!(world.stockpile.food, 50.0);
}

#[test]
fn research_uses_the_building_slot_reserves_once_and_enforces_prerequisites() {
    let mut world = GameWorld::default();
    world.stockpile.food = 100.0;
    world.stockpile.wood = 100.0;

    let before_missing_prerequisite = world.clone();
    assert_eq!(
        world.apply_command(Command::Research {
            building_id: "base-1".into(),
            technology: TechnologyKind::Mining,
        }),
        Err(CommandError::MissingTechnologyPrerequisite)
    );
    assert_eq!(world, before_missing_prerequisite);

    world
        .apply_command(Command::Research {
            building_id: "base-1".into(),
            technology: TechnologyKind::Masonry,
        })
        .unwrap();
    assert_eq!(world.stockpile.food, 60.0);
    assert_eq!(world.stockpile.wood, 80.0);

    let before_busy_rejection = world.clone();
    assert_eq!(
        world.apply_command(Command::Produce {
            building_id: "base-1".into(),
            product: ProductKind::Villager,
        }),
        Err(CommandError::BuildingBusy)
    );
    assert_eq!(world, before_busy_rejection);

    world.tick(RESEARCH_SECONDS - 0.1);
    assert!(world.researched_technologies.is_empty());
    world.tick(0.1);
    assert_eq!(world.researched_technologies, vec![TechnologyKind::Masonry]);
    assert_eq!(world.buildings[0].job, None);

    let before_duplicate = world.clone();
    assert_eq!(
        world.apply_command(Command::Research {
            building_id: "base-1".into(),
            technology: TechnologyKind::Masonry,
        }),
        Err(CommandError::TechnologyAlreadyResearched)
    );
    assert_eq!(world, before_duplicate);
}

#[test]
fn researched_abilities_make_their_resources_twenty_percent_faster() {
    for (technology, resource_kind) in [
        (TechnologyKind::Forestry, ResourceKind::Wood),
        (TechnologyKind::Agriculture, ResourceKind::Food),
        (TechnologyKind::Masonry, ResourceKind::Stone),
        (TechnologyKind::Masonry, ResourceKind::Clay),
        (TechnologyKind::Mining, ResourceKind::Gold),
        (TechnologyKind::Mining, ResourceKind::Iron),
        (TechnologyKind::Textiles, ResourceKind::Fiber),
    ] {
        let mut world = GameWorld {
            researched_technologies: vec![technology],
            ..GameWorld::default()
        };
        let index = world
            .resources
            .iter()
            .position(|resource| resource.kind == resource_kind)
            .unwrap();
        world.resources[index].amount = 20.0;
        world.units[0].position = world.resources[index].position;
        world
            .apply_command(Command::Gather {
                unit_id: "villager-1".into(),
                resource_id: world.resources[index].id.clone(),
            })
            .unwrap();
        world.tick(1.0);
        assert_eq!(
            world.resources[index].amount,
            20.0 - GATHER_RATE * GATHERING_TECH_MULTIPLIER,
            "{technology:?}"
        );
    }
}

#[test]
fn gatherer_waits_at_the_resource_until_the_load_is_full() {
    let mut world = GameWorld::default();
    start_gather_at_resource(&mut world, 0, 100.0);

    world.tick(4.0);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 8.0);
    assert_gather_phase(&world.units[0], GatherPhase::Gathering);
    assert_eq!(world.units[0].position, world.resources[0].position);

    world.tick(6.0);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 20.0);
    assert_gather_phase(&world.units[0], GatherPhase::Returning);
}

#[test]
fn simulation_speed_is_authoritative_validated_and_can_pause() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::SetSimulationSpeed { multiplier: 0.0 })
        .unwrap();
    let paused = world.clone();
    world.tick(100.0);
    assert_eq!(world, paused);

    world
        .apply_command(Command::SetSimulationSpeed { multiplier: 2.0 })
        .unwrap();
    start_gather_at_resource(&mut world, 0, 100.0);
    world.tick(1.0);
    assert_eq!(world.units[0].cargo.as_ref().unwrap().amount, 4.0);

    let before_invalid = world.clone();
    assert_eq!(
        world.apply_command(Command::SetSimulationSpeed { multiplier: 3.0 }),
        Err(CommandError::InvalidSimulationSpeed)
    );
    assert_eq!(world, before_invalid);
}

#[test]
fn busy_unit_rejects_commands_without_mutation() {
    let mut world = GameWorld::default();
    world
        .apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            resource_id: "tree-1".into(),
        })
        .unwrap();
    let before = world.clone();
    let error = world.apply_command(Command::Gather {
        unit_id: "villager-1".into(),
        resource_id: "tree-2".into(),
    });
    assert_eq!(error, Err(CommandError::UnitBusy));
    assert_eq!(world, before);
}

#[test]
fn build_reserves_once_works_for_four_seconds_and_creates_one_building() {
    let mut world = GameWorld::default();
    let initial_buildings = world.buildings.len();
    world.stockpile.wood = 20.0;
    world.units[0].position = Position { x: 500.0, y: 500.0 };
    world
        .apply_command(Command::Build {
            unit_id: "villager-1".into(),
            x: 500.0,
            y: 500.0,
        })
        .unwrap();
    assert_eq!(world.stockpile.wood, 0.0);

    world.tick(3.9);
    assert_eq!(world.buildings.len(), initial_buildings);
    assert!(matches!(world.units[0].action, UnitAction::Build { .. }));
    world.tick(0.1);
    assert_eq!(world.buildings.len(), initial_buildings + 1);
    assert_eq!(
        world.buildings.last().unwrap().kind,
        BuildingKind::TownCenter
    );
    assert_eq!(world.units[0].action, UnitAction::Idle);
    world.tick(10.0);
    assert_eq!(world.buildings.len(), initial_buildings + 1);
    assert_eq!(world.stockpile.wood, 0.0);
}

#[test]
fn build_validates_bounds_and_stockpile() {
    let mut world = GameWorld::default();
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            x: 100.0,
            y: 100.0,
        }),
        Err(CommandError::InsufficientWood)
    );
    world.stockpile.wood = 20.0;
    assert_eq!(
        world.apply_command(Command::Build {
            unit_id: "villager-1".into(),
            x: -1.0,
            y: 100.0,
        }),
        Err(CommandError::InvalidBuildSite)
    );
    assert_eq!(world.stockpile.wood, 20.0);
}
