use serde::{Deserialize, Serialize};

pub const WORLD_WIDTH: f64 = 1200.0;
pub const WORLD_HEIGHT: f64 = 800.0;
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
    pub tick: u64,
    pub units: Vec<Unit>,
    pub trees: Vec<Tree>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
    next_building_id: u64,
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
        Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            tick: 0,
            units: vec![
                Unit {
                    id: "villager-1".into(),
                    position: Position { x: 300.0, y: 400.0 },
                    action: UnitAction::Idle,
                },
                Unit {
                    id: "villager-2".into(),
                    position: Position { x: 400.0, y: 400.0 },
                    action: UnitAction::Idle,
                },
            ],
            trees: vec![
                tree("tree-1", 700.0, 200.0),
                tree("tree-2", 850.0, 300.0),
                tree("tree-3", 750.0, 550.0),
                tree("tree-4", 950.0, 600.0),
            ],
            buildings: Vec::new(),
            stockpile: Stockpile { wood: 0.0 },
            next_building_id: 1,
        }
    }
}

fn tree(id: &str, x: f64, y: f64) -> Tree {
    Tree {
        id: id.into(),
        position: Position { x, y },
        wood: 25.0,
    }
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
    use super::*;

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
