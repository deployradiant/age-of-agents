use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

pub const WORLD_WIDTH: f64 = 2400.0;
pub const WORLD_HEIGHT: f64 = 1600.0;
pub const CELL_SIZE: f64 = 80.0;
pub const WORLD_COLUMNS: u16 = 30;
pub const WORLD_ROWS: u16 = 20;
pub const UNIT_SIGHT_RADIUS: f64 = 320.0;
pub const BUILDING_SIGHT_RADIUS: f64 = 480.0;
pub const TOWN_CENTER_WOOD_COST: f64 = 20.0;
pub const BUILD_SECONDS: f64 = 4.0;
const MOVE_SPEED: f64 = 120.0;
const GATHER_RATE: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainBiome {
    Meadow,
    Forest,
    Prairie,
    Highland,
    Wetland,
    Scrubland,
    Heath,
    Clayland,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CellCoordinate {
    pub column: u16,
    pub row: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainCell {
    pub column: u16,
    pub row: u16,
    pub biome: TerrainBiome,
}

impl TerrainCell {
    fn coordinate(self) -> CellCoordinate {
        CellCoordinate {
            column: self.column,
            row: self.row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellVisibility {
    Unseen,
    Explored,
    Visible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotTerrainCell {
    pub column: u16,
    pub row: u16,
    pub biome: TerrainBiome,
    pub visibility: CellVisibility,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnitAction {
    Idle,
    Gather { tree_id: String },
    Build { x: f64, y: f64, work_seconds: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub position: Position,
    pub action: UnitAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tree {
    pub id: String,
    pub position: Position,
    pub wood: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub kind: BuildingKind,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingKind {
    TownCenter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stockpile {
    pub wood: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameWorld {
    pub width: f64,
    pub height: f64,
    pub cell_size: f64,
    pub tick: u64,
    pub terrain: Vec<TerrainCell>,
    pub explored_cells: Vec<CellCoordinate>,
    pub units: Vec<Unit>,
    pub trees: Vec<Tree>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
    next_building_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub width: f64,
    pub height: f64,
    pub cell_size: f64,
    pub tick: u64,
    pub terrain: Vec<SnapshotTerrainCell>,
    pub units: Vec<Unit>,
    pub trees: Vec<Tree>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Gather { unit_id: String, tree_id: String },
    Build { unit_id: String, x: f64, y: f64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    UnitNotFound,
    TreeNotFound,
    TreeDepleted,
    UnitBusy,
    InvalidBuildSite,
    InsufficientWood,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnitNotFound => "unit not found",
            Self::TreeNotFound => "tree not found",
            Self::TreeDepleted => "tree is depleted",
            Self::UnitBusy => "unit is busy",
            Self::InvalidBuildSite => "build site is outside the world",
            Self::InsufficientWood => "insufficient wood",
        };
        f.write_str(message)
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        let mut world = Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            cell_size: CELL_SIZE,
            tick: 0,
            terrain: generate_terrain(),
            explored_cells: Vec::new(),
            units: vec![
                Unit {
                    id: "villager-1".into(),
                    position: Position {
                        x: 1120.0,
                        y: 800.0,
                    },
                    action: UnitAction::Idle,
                },
                Unit {
                    id: "villager-2".into(),
                    position: Position {
                        x: 1200.0,
                        y: 800.0,
                    },
                    action: UnitAction::Idle,
                },
            ],
            trees: vec![
                tree("tree-1", 1360.0, 640.0),
                tree("tree-2", 1520.0, 800.0),
                tree("tree-3", 1320.0, 1040.0),
                tree("tree-4", 880.0, 1040.0),
            ],
            buildings: Vec::new(),
            stockpile: Stockpile { wood: 0.0 },
            next_building_id: 1,
        };
        world.refresh_exploration();
        world
    }
}

fn tree(id: &str, x: f64, y: f64) -> Tree {
    Tree {
        id: id.into(),
        position: Position { x, y },
        wood: 25.0,
    }
}

fn generate_terrain() -> Vec<TerrainCell> {
    const SITES: [(u16, u16, TerrainBiome); 8] = [
        (3, 3, TerrainBiome::Meadow),
        (11, 2, TerrainBiome::Forest),
        (21, 3, TerrainBiome::Prairie),
        (27, 7, TerrainBiome::Highland),
        (4, 14, TerrainBiome::Wetland),
        (12, 17, TerrainBiome::Scrubland),
        (20, 13, TerrainBiome::Heath),
        (27, 17, TerrainBiome::Clayland),
    ];

    let mut terrain = Vec::with_capacity(usize::from(WORLD_COLUMNS * WORLD_ROWS));
    for row in 0..WORLD_ROWS {
        for column in 0..WORLD_COLUMNS {
            let (_, _, biome) = SITES
                .iter()
                .min_by_key(|(site_column, site_row, _)| {
                    let dx = i32::from(column) - i32::from(*site_column);
                    let dy = i32::from(row) - i32::from(*site_row);
                    dx * dx + dy * dy
                })
                .expect("the fixed Voronoi map has sites");
            terrain.push(TerrainCell {
                column,
                row,
                biome: *biome,
            });
        }
    }
    terrain
}

impl GameWorld {
    pub fn apply_command(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::Gather { unit_id, tree_id } => {
                let unit_index = self.unit_index_and_idle(&unit_id)?;
                let tree = self
                    .trees
                    .iter()
                    .find(|tree| tree.id == tree_id)
                    .ok_or(CommandError::TreeNotFound)?;
                if tree.wood <= 0.0 {
                    return Err(CommandError::TreeDepleted);
                }
                self.units[unit_index].action = UnitAction::Gather { tree_id };
                Ok(())
            }
            Command::Build { unit_id, x, y } => {
                let unit_index = self.unit_index_and_idle(&unit_id)?;
                if !x.is_finite()
                    || !y.is_finite()
                    || !(0.0..=self.width).contains(&x)
                    || !(0.0..=self.height).contains(&y)
                {
                    return Err(CommandError::InvalidBuildSite);
                }
                if self.stockpile.wood < TOWN_CENTER_WOOD_COST {
                    return Err(CommandError::InsufficientWood);
                }
                self.stockpile.wood -= TOWN_CENTER_WOOD_COST;
                self.units[unit_index].action = UnitAction::Build {
                    x,
                    y,
                    work_seconds: 0.0,
                };
                Ok(())
            }
        }
    }

    fn unit_index_and_idle(&self, unit_id: &str) -> Result<usize, CommandError> {
        let index = self
            .units
            .iter()
            .position(|unit| unit.id == unit_id)
            .ok_or(CommandError::UnitNotFound)?;
        if self.units[index].action != UnitAction::Idle {
            return Err(CommandError::UnitBusy);
        }
        Ok(index)
    }

    pub fn tick(&mut self, dt: f64) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.tick += 1;
        for index in 0..self.units.len() {
            match self.units[index].action.clone() {
                UnitAction::Idle => {}
                UnitAction::Gather { tree_id } => self.tick_gather(index, tree_id, dt),
                UnitAction::Build { x, y, work_seconds } => {
                    self.tick_build(index, Position { x, y }, work_seconds, dt)
                }
            }
        }
        self.refresh_exploration();
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        let visible = self.visible_cells();
        let explored: BTreeSet<_> = self.explored_cells.iter().copied().collect();
        let is_visible = |position: Position| {
            self.cell_for_position(position)
                .is_some_and(|cell| visible.contains(&cell))
        };

        WorldSnapshot {
            width: self.width,
            height: self.height,
            cell_size: self.cell_size,
            tick: self.tick,
            terrain: self
                .terrain
                .iter()
                .map(|cell| {
                    let coordinate = cell.coordinate();
                    let visibility = if visible.contains(&coordinate) {
                        CellVisibility::Visible
                    } else if explored.contains(&coordinate) {
                        CellVisibility::Explored
                    } else {
                        CellVisibility::Unseen
                    };
                    SnapshotTerrainCell {
                        column: cell.column,
                        row: cell.row,
                        biome: cell.biome,
                        visibility,
                    }
                })
                .collect(),
            units: self
                .units
                .iter()
                .filter(|unit| is_visible(unit.position))
                .cloned()
                .collect(),
            trees: self
                .trees
                .iter()
                .filter(|tree| is_visible(tree.position))
                .cloned()
                .collect(),
            buildings: self
                .buildings
                .iter()
                .filter(|building| is_visible(building.position))
                .cloned()
                .collect(),
            stockpile: self.stockpile.clone(),
        }
    }

    fn refresh_exploration(&mut self) {
        let visible = self.visible_cells();
        let mut explored: BTreeSet<_> = self
            .explored_cells
            .iter()
            .copied()
            .filter(|cell| cell.column < WORLD_COLUMNS && cell.row < WORLD_ROWS)
            .collect();
        explored.extend(visible);
        self.explored_cells = explored.into_iter().collect();
    }

    fn visible_cells(&self) -> BTreeSet<CellCoordinate> {
        let mut visible = BTreeSet::new();
        for cell in &self.terrain {
            let center = Position {
                x: (f64::from(cell.column) + 0.5) * self.cell_size,
                y: (f64::from(cell.row) + 0.5) * self.cell_size,
            };
            let seen_by_unit = self
                .units
                .iter()
                .any(|unit| unit.position.distance(center) <= UNIT_SIGHT_RADIUS);
            let seen_by_building = self
                .buildings
                .iter()
                .any(|building| building.position.distance(center) <= BUILDING_SIGHT_RADIUS);
            if seen_by_unit || seen_by_building {
                visible.insert(cell.coordinate());
            }
        }
        visible
    }

    fn cell_for_position(&self, position: Position) -> Option<CellCoordinate> {
        if !position.x.is_finite()
            || !position.y.is_finite()
            || !(0.0..=self.width).contains(&position.x)
            || !(0.0..=self.height).contains(&position.y)
        {
            return None;
        }
        let column = ((position.x / self.cell_size).floor() as u16).min(WORLD_COLUMNS - 1);
        let row = ((position.y / self.cell_size).floor() as u16).min(WORLD_ROWS - 1);
        Some(CellCoordinate { column, row })
    }

    fn tick_gather(&mut self, unit_index: usize, tree_id: String, dt: f64) {
        let Some(tree_index) = self.trees.iter().position(|tree| tree.id == tree_id) else {
            self.units[unit_index].action = UnitAction::Idle;
            return;
        };
        if self.trees[tree_index].wood <= 0.0 {
            self.units[unit_index].action = UnitAction::Idle;
            return;
        }

        let target = self.trees[tree_index].position;
        let remaining = move_toward(&mut self.units[unit_index].position, target, dt);
        if remaining <= 0.0 {
            return;
        }
        let gathered = (GATHER_RATE * remaining).min(self.trees[tree_index].wood);
        self.trees[tree_index].wood -= gathered;
        self.stockpile.wood += gathered;
        if self.trees[tree_index].wood <= f64::EPSILON {
            self.trees[tree_index].wood = 0.0;
            self.units[unit_index].action = UnitAction::Idle;
        }
    }

    fn tick_build(&mut self, unit_index: usize, target: Position, work_seconds: f64, dt: f64) {
        let remaining = move_toward(&mut self.units[unit_index].position, target, dt);
        let completed_work = work_seconds + remaining;
        if completed_work + f64::EPSILON < BUILD_SECONDS {
            self.units[unit_index].action = UnitAction::Build {
                x: target.x,
                y: target.y,
                work_seconds: completed_work,
            };
            return;
        }

        self.buildings.push(Building {
            id: format!("building-{}", self.next_building_id),
            kind: BuildingKind::TownCenter,
            position: target,
        });
        self.next_building_id += 1;
        self.units[unit_index].action = UnitAction::Idle;
    }
}

/// Moves for up to `dt`, returning any time left after reaching the target.
fn move_toward(position: &mut Position, target: Position, dt: f64) -> f64 {
    let distance = position.distance(target);
    if distance <= f64::EPSILON {
        return dt;
    }
    let travel_seconds = distance / MOVE_SPEED;
    if travel_seconds <= dt {
        *position = target;
        dt - travel_seconds
    } else {
        let fraction = dt / travel_seconds;
        position.x += (target.x - position.x) * fraction;
        position.y += (target.y - position.y) * fraction;
        0.0
    }
}

#[cfg(test)]
mod tests {
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
    fn snapshots_have_three_typed_fog_states_and_hide_unseen_trees() {
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
        assert!(snapshot.trees.is_empty());
        assert_eq!(snapshot.units.len(), 2);
    }

    #[test]
    fn completed_buildings_reveal_with_the_larger_building_radius() {
        let mut world = GameWorld::default();
        world.units.clear();
        world.buildings.push(Building {
            id: "building-sight".into(),
            kind: BuildingKind::TownCenter,
            position: Position {
                x: 1200.0,
                y: 800.0,
            },
        });
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

    #[test]
    fn gather_moves_collects_directly_and_finishes_on_depletion() {
        let mut world = GameWorld::default();
        world.units[0].position = world.trees[0].position;
        world.trees[0].wood = 15.0;
        world
            .apply_command(Command::Gather {
                unit_id: "villager-1".into(),
                tree_id: "tree-1".into(),
            })
            .unwrap();

        world.tick(1.0);
        assert_eq!(world.stockpile.wood, 10.0);
        assert!(matches!(world.units[0].action, UnitAction::Gather { .. }));
        world.tick(0.5);
        assert_eq!(world.stockpile.wood, 15.0);
        assert_eq!(world.trees[0].wood, 0.0);
        assert_eq!(world.units[0].action, UnitAction::Idle);
    }

    #[test]
    fn busy_unit_rejects_commands_without_mutation() {
        let mut world = GameWorld::default();
        world
            .apply_command(Command::Gather {
                unit_id: "villager-1".into(),
                tree_id: "tree-1".into(),
            })
            .unwrap();
        let before = world.clone();
        let error = world.apply_command(Command::Gather {
            unit_id: "villager-1".into(),
            tree_id: "tree-2".into(),
        });
        assert_eq!(error, Err(CommandError::UnitBusy));
        assert_eq!(world, before);
    }

    #[test]
    fn build_reserves_once_works_for_four_seconds_and_creates_one_building() {
        let mut world = GameWorld::default();
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
        assert!(world.buildings.is_empty());
        assert!(matches!(world.units[0].action, UnitAction::Build { .. }));
        world.tick(0.1);
        assert_eq!(world.buildings.len(), 1);
        assert_eq!(world.buildings[0].kind, BuildingKind::TownCenter);
        assert_eq!(world.units[0].action, UnitAction::Idle);
        world.tick(10.0);
        assert_eq!(world.buildings.len(), 1);
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
}
