use std::collections::BTreeSet;

use super::*;

#[test]
fn group_move_assigns_distinct_deterministic_reachable_destinations() {
    let mut world = GameWorld::default();
    world.resources.clear();
    world.buildings.clear();
    let target = world.cell_center(CellCoordinate {
        column: 10,
        row: 10,
    });
    world
        .apply_command(Command::GroupMove {
            unit_ids: vec!["villager-2".into(), "villager-1".into()],
            x: target.x,
            y: target.y,
        })
        .unwrap();
    let destinations: BTreeSet<_> = world
        .units
        .iter()
        .map(|unit| match unit.action {
            UnitAction::Move { x, y } => world.cell_for_position(Position { x, y }).unwrap(),
            _ => panic!("group member did not receive a move"),
        })
        .collect();
    assert_eq!(destinations.len(), 2);
    assert!(destinations.contains(&CellCoordinate {
        column: 10,
        row: 10
    }));
    let starts: Vec<_> = world.units.iter().map(|unit| unit.position).collect();
    for _ in 0..600 {
        world.tick(0.05);
        let occupied: BTreeSet<_> = world
            .units
            .iter()
            .map(|unit| world.cell_for_position(unit.position).unwrap())
            .collect();
        assert_eq!(
            occupied.len(),
            world.units.len(),
            "group members stacked at tick {}",
            world.tick
        );
        if world
            .units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle)
        {
            break;
        }
    }
    assert!(
        world
            .units
            .iter()
            .zip(starts)
            .all(|(unit, start)| unit.position != start)
    );
    assert!(
        world
            .units
            .iter()
            .all(|unit| unit.action == UnitAction::Idle)
    );
}

#[test]
fn group_move_rejections_are_atomic_for_duplicate_unknown_and_busy_members() {
    for (unit_ids, expected) in [
        (
            vec!["villager-1".into(), "villager-1".into()],
            CommandError::DuplicateUnit,
        ),
        (
            vec!["villager-1".into(), "missing".into()],
            CommandError::UnitNotFound,
        ),
    ] {
        let mut world = GameWorld::default();
        let before = world.clone();
        assert_eq!(
            world.apply_command(Command::GroupMove {
                unit_ids,
                x: 400.0,
                y: 400.0
            }),
            Err(expected)
        );
        assert_eq!(world, before);
    }
    let mut world = GameWorld::default();
    world.units[1].action = UnitAction::Move { x: 400.0, y: 400.0 };
    let before = world.clone();
    assert_eq!(
        world.apply_command(Command::GroupMove {
            unit_ids: vec!["villager-1".into(), "villager-2".into()],
            x: 600.0,
            y: 600.0,
        }),
        Err(CommandError::UnitBusy)
    );
    assert_eq!(world, before);
}
