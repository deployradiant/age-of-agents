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
pub const VILLAGER_FOOD_COST: f64 = 50.0;
pub const VILLAGER_PRODUCTION_SECONDS: f64 = 6.0;
const MOVE_SPEED: f64 = 120.0;
const GATHER_RATE: f64 = 10.0;
pub const VILLAGER_CARRY_CAPACITY: f64 = 20.0;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biome: Option<TerrainBiome>,
    pub visibility: CellVisibility,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnitAction {
    Idle,
    Move {
        x: f64,
        y: f64,
    },
    Gather {
        resource_id: String,
        #[serde(default)]
        phase: GatherPhase,
    },
    Build {
        x: f64,
        y: f64,
        work_seconds: f64,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatherPhase {
    #[default]
    ToResource,
    Gathering,
    Returning,
    Depositing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CarriedResource {
    pub kind: ResourceKind,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub position: Position,
    pub action: UnitAction,
    #[serde(default)]
    pub cargo: Option<CarriedResource>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: String,
    pub kind: ResourceKind,
    pub position: Position,
    pub amount: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Wood,
    Food,
    Stone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub kind: BuildingKind,
    pub position: Position,
    pub produces: Vec<ProductKind>,
    pub production: Option<ProductionJob>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingKind {
    TownCenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductKind {
    Villager,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductionJob {
    pub product: ProductKind,
    pub elapsed_seconds: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stockpile {
    pub wood: f64,
    pub food: f64,
    pub stone: f64,
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
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
    next_building_id: u64,
    next_unit_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub width: f64,
    pub height: f64,
    pub cell_size: f64,
    pub tick: u64,
    pub terrain: Vec<SnapshotTerrainCell>,
    pub units: Vec<Unit>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Move {
        unit_id: String,
        x: f64,
        y: f64,
    },
    Gather {
        unit_id: String,
        resource_id: String,
    },
    Build {
        unit_id: String,
        x: f64,
        y: f64,
    },
    Produce {
        building_id: String,
        product: ProductKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    UnitNotFound,
    ResourceNotFound,
    ResourceDepleted,
    UnitBusy,
    InvalidDestination,
    InvalidBuildSite,
    InsufficientWood,
    BuildingNotFound,
    BuildingBusy,
    ProductUnavailable,
    InsufficientFood,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnitNotFound => "unit not found",
            Self::ResourceNotFound => "resource not found",
            Self::ResourceDepleted => "resource is depleted",
            Self::UnitBusy => "unit is busy",
            Self::InvalidDestination => "destination is outside the world",
            Self::InvalidBuildSite => "build site is outside the world",
            Self::InsufficientWood => "insufficient wood",
            Self::BuildingNotFound => "building not found",
            Self::BuildingBusy => "building is already producing",
            Self::ProductUnavailable => "building cannot produce that item",
            Self::InsufficientFood => "insufficient food",
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
                        x: 1080.0,
                        y: 880.0,
                    },
                    action: UnitAction::Idle,
                    cargo: None,
                },
                Unit {
                    id: "villager-2".into(),
                    position: Position {
                        x: 1240.0,
                        y: 880.0,
                    },
                    action: UnitAction::Idle,
                    cargo: None,
                },
            ],
            resources: vec![
                resource("tree-1", ResourceKind::Wood, 1360.0, 640.0, 25.0),
                resource("tree-2", ResourceKind::Wood, 1520.0, 800.0, 25.0),
                resource("tree-3", ResourceKind::Wood, 1320.0, 1040.0, 25.0),
                resource("tree-4", ResourceKind::Wood, 880.0, 1040.0, 25.0),
                resource("tree-5", ResourceKind::Wood, 760.0, 720.0, 25.0),
                resource("tree-6", ResourceKind::Wood, 1680.0, 1040.0, 25.0),
                resource("berries-1", ResourceKind::Food, 960.0, 640.0, 50.0),
                resource("berries-2", ResourceKind::Food, 1040.0, 1120.0, 50.0),
                resource("berries-3", ResourceKind::Food, 1440.0, 1120.0, 50.0),
                resource("berries-4", ResourceKind::Food, 720.0, 1200.0, 50.0),
                resource("stone-1", ResourceKind::Stone, 840.0, 800.0, 40.0),
                resource("stone-2", ResourceKind::Stone, 1600.0, 640.0, 40.0),
                resource("stone-3", ResourceKind::Stone, 1520.0, 1200.0, 40.0),
                resource("stone-4", ResourceKind::Stone, 640.0, 880.0, 40.0),
            ],
            buildings: vec![town_center("base-1", 1160.0, 720.0)],
            stockpile: Stockpile {
                wood: 0.0,
                food: 0.0,
                stone: 0.0,
            },
            next_building_id: 2,
            next_unit_id: 3,
        };
        world.refresh_exploration();
        world
    }
}

fn resource(id: &str, kind: ResourceKind, x: f64, y: f64, amount: f64) -> ResourceNode {
    ResourceNode {
        id: id.into(),
        kind,
        position: Position { x, y },
        amount,
    }
}

fn town_center(id: &str, x: f64, y: f64) -> Building {
    Building {
        id: id.into(),
        kind: BuildingKind::TownCenter,
        position: Position { x, y },
        produces: vec![ProductKind::Villager],
        production: None,
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
            Command::Move { unit_id, x, y } => {
                let unit_index = self.unit_index_and_idle(&unit_id)?;
                if !self.contains_position(x, y) {
                    return Err(CommandError::InvalidDestination);
                }
                self.units[unit_index].action = UnitAction::Move { x, y };
                Ok(())
            }
            Command::Gather {
                unit_id,
                resource_id,
            } => {
                let unit_index = self.unit_index_and_idle(&unit_id)?;
                let resource = self
                    .resources
                    .iter()
                    .find(|resource| resource.id == resource_id)
                    .ok_or(CommandError::ResourceNotFound)?;
                if resource.amount <= 0.0 {
                    return Err(CommandError::ResourceDepleted);
                }
                self.units[unit_index].action = UnitAction::Gather {
                    resource_id,
                    phase: GatherPhase::ToResource,
                };
                Ok(())
            }
            Command::Build { unit_id, x, y } => {
                let unit_index = self.unit_index_and_idle(&unit_id)?;
                if !self.contains_position(x, y) {
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
            Command::Produce {
                building_id,
                product,
            } => {
                let building_index = self
                    .buildings
                    .iter()
                    .position(|building| building.id == building_id)
                    .ok_or(CommandError::BuildingNotFound)?;
                let building = &self.buildings[building_index];
                if !building.produces.contains(&product) {
                    return Err(CommandError::ProductUnavailable);
                }
                if building.production.is_some() {
                    return Err(CommandError::BuildingBusy);
                }
                match product {
                    ProductKind::Villager if self.stockpile.food < VILLAGER_FOOD_COST => {
                        return Err(CommandError::InsufficientFood);
                    }
                    ProductKind::Villager => self.stockpile.food -= VILLAGER_FOOD_COST,
                }
                self.buildings[building_index].production = Some(ProductionJob {
                    product,
                    elapsed_seconds: 0.0,
                });
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

    fn contains_position(&self, x: f64, y: f64) -> bool {
        x.is_finite()
            && y.is_finite()
            && (0.0..=self.width).contains(&x)
            && (0.0..=self.height).contains(&y)
    }

    pub fn tick(&mut self, dt: f64) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        self.tick += 1;
        for index in 0..self.units.len() {
            match self.units[index].action.clone() {
                UnitAction::Idle => {}
                UnitAction::Move { x, y } => self.tick_move(index, Position { x, y }, dt),
                UnitAction::Gather { resource_id, phase } => {
                    self.tick_gather(index, resource_id, phase, dt)
                }
                UnitAction::Build { x, y, work_seconds } => {
                    self.tick_build(index, Position { x, y }, work_seconds, dt)
                }
            }
        }
        for index in 0..self.buildings.len() {
            self.tick_production(index, dt);
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
        let is_discovered = |position: Position| {
            self.cell_for_position(position)
                .is_some_and(|cell| explored.contains(&cell))
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
                        biome: (visibility != CellVisibility::Unseen).then_some(cell.biome),
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
            resources: self
                .resources
                .iter()
                .filter(|resource| is_discovered(resource.position))
                .cloned()
                .collect(),
            buildings: self
                .buildings
                .iter()
                .filter(|building| is_discovered(building.position))
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

    fn tick_move(&mut self, unit_index: usize, target: Position, dt: f64) {
        move_toward(&mut self.units[unit_index].position, target, dt);
        if self.units[unit_index].position.distance(target) <= f64::EPSILON {
            self.units[unit_index].action = UnitAction::Idle;
        }
    }

    fn tick_gather(&mut self, unit_index: usize, resource_id: String, phase: GatherPhase, dt: f64) {
        match phase {
            GatherPhase::ToResource => self.tick_to_resource(unit_index, resource_id, dt),
            GatherPhase::Gathering => self.tick_at_resource(unit_index, resource_id, dt),
            GatherPhase::Returning => self.tick_returning(unit_index, resource_id, dt),
            GatherPhase::Depositing => self.tick_depositing(unit_index, resource_id),
        }
    }

    fn tick_to_resource(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        let Some(resource_index) = self
            .resources
            .iter()
            .position(|resource| resource.id == resource_id)
        else {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        };
        if self.resources[resource_index].amount <= 0.0 {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        }
        if self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.amount + f64::EPSILON >= VILLAGER_CARRY_CAPACITY)
        {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }

        let target = self.resources[resource_index].position;
        let remaining = move_toward(&mut self.units[unit_index].position, target, dt);
        if self.units[unit_index].position.distance(target) > f64::EPSILON {
            return;
        }
        self.set_gather_phase(unit_index, resource_id.clone(), GatherPhase::Gathering);
        if remaining > 0.0 {
            self.tick_at_resource(unit_index, resource_id, remaining);
        }
    }

    fn tick_at_resource(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        let Some(resource_index) = self
            .resources
            .iter()
            .position(|resource| resource.id == resource_id)
        else {
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        };
        if self.resources[resource_index].amount <= f64::EPSILON {
            self.resources[resource_index].amount = 0.0;
            self.finish_or_return_with_cargo(unit_index, resource_id);
            return;
        }

        let kind = self.resources[resource_index].kind;
        if self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.kind != kind)
        {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }
        let carried = self.units[unit_index]
            .cargo
            .as_ref()
            .map_or(0.0, |cargo| cargo.amount);
        let capacity_left = (VILLAGER_CARRY_CAPACITY - carried).max(0.0);
        let gathered = (GATHER_RATE * dt)
            .min(self.resources[resource_index].amount)
            .min(capacity_left);
        self.resources[resource_index].amount -= gathered;
        if gathered > 0.0 {
            let cargo = self.units[unit_index]
                .cargo
                .get_or_insert(CarriedResource { kind, amount: 0.0 });
            cargo.amount += gathered;
        }
        if self.resources[resource_index].amount <= f64::EPSILON {
            self.resources[resource_index].amount = 0.0;
        }
        let full = self.units[unit_index]
            .cargo
            .as_ref()
            .is_some_and(|cargo| cargo.amount + f64::EPSILON >= VILLAGER_CARRY_CAPACITY);
        if full || self.resources[resource_index].amount == 0.0 {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
        }
    }

    fn tick_returning(&mut self, unit_index: usize, resource_id: String, dt: f64) {
        if self.units[unit_index].cargo.is_none() {
            self.resume_or_finish_gather(unit_index, resource_id);
            return;
        }
        let Some(target) = self.nearest_town_center(self.units[unit_index].position) else {
            return;
        };
        move_toward(&mut self.units[unit_index].position, target, dt);
        if self.units[unit_index].position.distance(target) <= f64::EPSILON {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Depositing);
        }
    }

    fn tick_depositing(&mut self, unit_index: usize, resource_id: String) {
        let Some(target) = self.nearest_town_center(self.units[unit_index].position) else {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        };
        if self.units[unit_index].position.distance(target) > f64::EPSILON {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
            return;
        }
        if let Some(cargo) = self.units[unit_index].cargo.take() {
            match cargo.kind {
                ResourceKind::Wood => self.stockpile.wood += cargo.amount,
                ResourceKind::Food => self.stockpile.food += cargo.amount,
                ResourceKind::Stone => self.stockpile.stone += cargo.amount,
            }
        }
        self.resume_or_finish_gather(unit_index, resource_id);
    }

    fn finish_or_return_with_cargo(&mut self, unit_index: usize, resource_id: String) {
        if self.units[unit_index].cargo.is_some() {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::Returning);
        } else {
            self.units[unit_index].action = UnitAction::Idle;
        }
    }

    fn resume_or_finish_gather(&mut self, unit_index: usize, resource_id: String) {
        let resource_remains = self
            .resources
            .iter()
            .any(|resource| resource.id == resource_id && resource.amount > f64::EPSILON);
        if resource_remains {
            self.set_gather_phase(unit_index, resource_id, GatherPhase::ToResource);
        } else {
            self.units[unit_index].action = UnitAction::Idle;
        }
    }

    fn set_gather_phase(&mut self, unit_index: usize, resource_id: String, phase: GatherPhase) {
        self.units[unit_index].action = UnitAction::Gather { resource_id, phase };
    }

    fn nearest_town_center(&self, position: Position) -> Option<Position> {
        self.buildings
            .iter()
            .filter(|building| building.kind == BuildingKind::TownCenter)
            .min_by(|left, right| {
                position
                    .distance(left.position)
                    .total_cmp(&position.distance(right.position))
                    .then_with(|| left.id.cmp(&right.id))
            })
            .map(|building| building.position)
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

        let building_id = format!("building-{}", self.next_building_id);
        self.buildings
            .push(town_center(&building_id, target.x, target.y));
        self.next_building_id += 1;
        self.units[unit_index].action = UnitAction::Idle;
    }

    fn tick_production(&mut self, building_index: usize, dt: f64) {
        let Some(mut job) = self.buildings[building_index].production.clone() else {
            return;
        };
        job.elapsed_seconds += dt;
        let required_seconds = match job.product {
            ProductKind::Villager => VILLAGER_PRODUCTION_SECONDS,
        };
        if job.elapsed_seconds + f64::EPSILON < required_seconds {
            self.buildings[building_index].production = Some(job);
            return;
        }

        let building_position = self.buildings[building_index].position;
        match job.product {
            ProductKind::Villager => {
                self.units.push(Unit {
                    id: format!("villager-{}", self.next_unit_id),
                    position: Position {
                        x: (building_position.x + 100.0).min(self.width),
                        y: (building_position.y + 80.0).min(self.height),
                    },
                    action: UnitAction::Idle,
                    cargo: None,
                });
                self.next_unit_id += 1;
            }
        }
        self.buildings[building_index].production = None;
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
mod tests;
