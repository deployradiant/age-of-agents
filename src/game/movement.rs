use std::collections::BTreeSet;

use super::*;
use crate::navigation::{find_path, neighbors};

impl GameWorld {
    pub(super) fn cell_center(&self, cell: CellCoordinate) -> Position {
        Position {
            x: (f64::from(cell.column) + 0.5) * self.cell_size,
            y: (f64::from(cell.row) + 0.5) * self.cell_size,
        }
    }

    fn occupied_cells(&self, moving_unit: Option<usize>) -> BTreeSet<CellCoordinate> {
        let mut occupied = BTreeSet::new();
        for (index, unit) in self.units.iter().enumerate() {
            if Some(index) != moving_unit {
                occupied.extend(self.cell_for_position(unit.position));
                if let UnitAction::Move { x, y } = unit.action {
                    occupied.extend(self.cell_for_position(Position { x, y }));
                }
            }
            if let UnitAction::Build { x, y, .. } = unit.action {
                occupied.extend(self.cell_for_position(Position { x, y }));
            }
        }
        occupied.extend(
            self.resources
                .iter()
                .filter(|resource| resource.amount > f64::EPSILON)
                .filter_map(|resource| self.cell_for_position(resource.position)),
        );
        occupied.extend(
            self.buildings
                .iter()
                .filter_map(|building| self.cell_for_position(building.position)),
        );
        occupied
    }

    fn route(
        &self,
        unit_index: usize,
        goal: CellCoordinate,
        blocked: &BTreeSet<CellCoordinate>,
    ) -> Option<Vec<CellCoordinate>> {
        let start = self.cell_for_position(self.units[unit_index].position)?;
        find_path(WORLD_COLUMNS, WORLD_ROWS, start, goal, blocked)
    }

    fn interaction_route(
        &self,
        unit_index: usize,
        occupied_target: CellCoordinate,
    ) -> Option<(Position, Vec<CellCoordinate>)> {
        let start = self.cell_for_position(self.units[unit_index].position)?;
        if start == occupied_target {
            return Some((self.units[unit_index].position, Vec::new()));
        }
        let blocked = self.occupied_cells(Some(unit_index));
        neighbors(occupied_target, WORLD_COLUMNS, WORLD_ROWS)
            .filter(|cell| !blocked.contains(cell))
            .filter_map(|goal| {
                find_path(WORLD_COLUMNS, WORLD_ROWS, start, goal, &blocked)
                    .map(|route| (route.len(), goal, route))
            })
            .min_by_key(|(length, goal, _)| (*length, *goal))
            .map(|(_, goal, route)| (self.cell_center(goal), route))
    }

    fn tick_toward_free_cell(
        &mut self,
        unit_index: usize,
        target: Position,
        dt: f64,
    ) -> (bool, f64) {
        let Some(goal) = self.cell_for_position(target) else {
            return (false, 0.0);
        };
        let blocked = self.occupied_cells(Some(unit_index));
        let Some(route) = self.route(unit_index, goal, &blocked) else {
            return (false, 0.0);
        };
        let mut waypoints: Vec<_> = route
            .into_iter()
            .map(|cell| self.cell_center(cell))
            .collect();
        if waypoints.last().is_none_or(|waypoint| *waypoint != target) {
            waypoints.push(target);
        }
        let remaining = move_along(&mut self.units[unit_index].position, waypoints, dt);
        (self.units[unit_index].position == target, remaining)
    }

    fn tick_toward_occupied_cell(
        &mut self,
        unit_index: usize,
        target: Position,
        dt: f64,
    ) -> (bool, f64) {
        let Some(target_cell) = self.cell_for_position(target) else {
            return (false, 0.0);
        };
        let Some((destination, route)) = self.interaction_route(unit_index, target_cell) else {
            return (false, 0.0);
        };
        let waypoints: Vec<_> = route
            .into_iter()
            .map(|cell| self.cell_center(cell))
            .collect();
        let remaining = move_along(&mut self.units[unit_index].position, waypoints, dt);
        (self.units[unit_index].position == destination, remaining)
    }

    pub(super) fn tick_move(&mut self, unit_index: usize, target: Position, dt: f64) {
        if self.tick_toward_free_cell(unit_index, target, dt).0 {
            self.units[unit_index].action = UnitAction::Idle;
        }
    }

    pub(super) fn route_exists_to_free_cell(
        &self,
        unit_index: usize,
        target: Position,
    ) -> Result<(), CommandError> {
        let target_cell = self
            .cell_for_position(target)
            .ok_or(CommandError::InvalidDestination)?;
        let blocked = self.occupied_cells(Some(unit_index));
        if blocked.contains(&target_cell) {
            return Err(CommandError::DestinationOccupied);
        }
        self.route(unit_index, target_cell, &blocked)
            .map(|_| ())
            .ok_or(CommandError::TargetUnreachable)
    }

    pub(super) fn interaction_route_exists(&self, unit_index: usize, target: Position) -> bool {
        self.cell_for_position(target)
            .and_then(|cell| self.interaction_route(unit_index, cell))
            .is_some()
    }

    pub(super) fn site_is_occupied(&self, site: Position) -> bool {
        self.cell_for_position(site)
            .is_none_or(|cell| self.occupied_cells(None).contains(&cell))
    }

    pub(super) fn adjacent_to(&self, unit_index: usize, target: Position) -> bool {
        let Some(target) = self.cell_for_position(target) else {
            return false;
        };
        let Some(unit) = self.cell_for_position(self.units[unit_index].position) else {
            return false;
        };
        unit == target || neighbors(target, WORLD_COLUMNS, WORLD_ROWS).any(|cell| cell == unit)
    }

    pub(super) fn nearest_reachable_town_center(&self, unit_index: usize) -> Option<Position> {
        self.buildings
            .iter()
            .filter(|building| building.kind == BuildingKind::TownCenter)
            .filter_map(|building| {
                let cell = self.cell_for_position(building.position)?;
                let (_, route) = self.interaction_route(unit_index, cell)?;
                Some((route.len(), &building.id, building.position))
            })
            .min_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)))
            .map(|(_, _, position)| position)
    }

    pub(super) fn spawn_position(&self, building_index: usize) -> Option<Position> {
        let building_cell = self.cell_for_position(self.buildings[building_index].position)?;
        let occupied = self.occupied_cells(None);
        neighbors(building_cell, WORLD_COLUMNS, WORLD_ROWS)
            .find(|cell| !occupied.contains(cell))
            .map(|cell| self.cell_center(cell))
    }

    pub(super) fn tick_toward_interaction(
        &mut self,
        unit_index: usize,
        target: Position,
        dt: f64,
    ) -> (bool, f64) {
        self.tick_toward_occupied_cell(unit_index, target, dt)
    }
}
