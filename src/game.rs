use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

mod domain;
mod gathering;
#[cfg(test)]
mod group_move_tests;
mod movement;
#[cfg(test)]
mod slice_a_tests;

pub use domain::*;

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
pub const RESEARCH_FOOD_COST: f64 = 40.0;
pub const RESEARCH_WOOD_COST: f64 = 20.0;
pub const RESEARCH_SECONDS: f64 = 8.0;
pub const GATHERING_TECH_MULTIPLIER: f64 = 1.2;
pub const RESOURCE_MIN_SEPARATION: f64 = 120.0;
pub const STARTING_BASE_RESOURCE_CLEARANCE: f64 = 200.0;
const MOVE_SPEED: f64 = 120.0;
pub(crate) const GATHER_RATE: f64 = 2.0;
pub const VILLAGER_CARRY_CAPACITY: f64 = 20.0;

fn default_simulation_speed() -> f64 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameWorld {
    pub width: f64,
    pub height: f64,
    pub cell_size: f64,
    pub tick: u64,
    #[serde(default = "default_simulation_speed")]
    pub simulation_speed: f64,
    pub terrain: Vec<TerrainCell>,
    pub explored_cells: Vec<CellCoordinate>,
    pub units: Vec<Unit>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
    pub researched_technologies: Vec<TechnologyKind>,
    #[serde(default)]
    pub scenario: ScenarioState,
    next_building_id: u64,
    next_unit_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WorldSnapshot {
    pub width: f64,
    pub height: f64,
    pub cell_size: f64,
    pub tick: u64,
    pub simulation_speed: f64,
    pub terrain: Vec<SnapshotTerrainCell>,
    pub units: Vec<Unit>,
    pub resources: Vec<ResourceNode>,
    pub buildings: Vec<Building>,
    pub stockpile: Stockpile,
    pub researched_technologies: Vec<TechnologyKind>,
    pub catalog: DomainCatalog,
    pub scenario: ScenarioState,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Move {
        unit_id: String,
        x: f64,
        y: f64,
    },
    GroupMove {
        unit_ids: Vec<String>,
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
    Research {
        building_id: String,
        technology: TechnologyKind,
    },
    SetSimulationSpeed {
        multiplier: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    UnitNotFound,
    EmptyUnitGroup,
    DuplicateUnit,
    ResourceNotFound,
    ResourceDepleted,
    UnitBusy,
    InvalidDestination,
    DestinationOccupied,
    TargetUnreachable,
    InvalidBuildSite,
    InsufficientWood,
    BuildingNotFound,
    BuildingBusy,
    ProductUnavailable,
    InsufficientFood,
    TechnologyUnavailable,
    TechnologyAlreadyResearched,
    MissingTechnologyPrerequisite,
    InsufficientResearchResources,
    InvalidSimulationSpeed,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::UnitNotFound => "unit not found",
            Self::EmptyUnitGroup => "unit group is empty",
            Self::DuplicateUnit => "unit group contains a duplicate member",
            Self::ResourceNotFound => "resource not found",
            Self::ResourceDepleted => "resource is depleted",
            Self::UnitBusy => "unit is busy",
            Self::InvalidDestination => "destination is outside the world",
            Self::DestinationOccupied => "destination cell is occupied",
            Self::TargetUnreachable => "target is unreachable",
            Self::InvalidBuildSite => "build site is outside the world",
            Self::InsufficientWood => "insufficient wood",
            Self::BuildingNotFound => "building not found",
            Self::BuildingBusy => "building is already producing",
            Self::ProductUnavailable => "building cannot produce that item",
            Self::InsufficientFood => "insufficient food",
            Self::TechnologyUnavailable => "building cannot research that technology",
            Self::TechnologyAlreadyResearched => "technology is already researched",
            Self::MissingTechnologyPrerequisite => "technology prerequisite is not researched",
            Self::InsufficientResearchResources => "research requires 40 food and 20 wood",
            Self::InvalidSimulationSpeed => "simulation speed must be 0, 1, or 2",
        };
        f.write_str(message)
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        let terrain = generate_terrain();
        let mut world = Self {
            width: WORLD_WIDTH,
            height: WORLD_HEIGHT,
            cell_size: CELL_SIZE,
            tick: 0,
            simulation_speed: default_simulation_speed(),
            terrain: terrain.clone(),
            explored_cells: Vec::new(),
            units: vec![
                Unit {
                    id: "villager-1".into(),
                    kind: UnitKind::Villager,
                    position: Position {
                        x: 1080.0,
                        y: 880.0,
                    },
                    action: UnitAction::Idle,
                    cargo: None,
                },
                Unit {
                    id: "villager-2".into(),
                    kind: UnitKind::Villager,
                    position: Position {
                        x: 1240.0,
                        y: 880.0,
                    },
                    action: UnitAction::Idle,
                    cargo: None,
                },
            ],
            resources: generate_resources(&terrain),
            buildings: vec![town_center("base-1", 1160.0, 720.0)],
            stockpile: Stockpile {
                wood: 0.0,
                food: 0.0,
                stone: 0.0,
                gold: 0.0,
                iron: 0.0,
                coal: 0.0,
                clay: 0.0,
                fiber: 0.0,
                timber: 0.0,
                steel: 0.0,
                bricks: 0.0,
                cloth: 0.0,
                rations: 0.0,
            },
            researched_technologies: Vec::new(),
            scenario: ScenarioState::default(),
            next_building_id: 2,
            next_unit_id: 3,
        };
        world.refresh_exploration();
        world
    }
}

fn town_center(id: &str, x: f64, y: f64) -> Building {
    Building {
        id: id.into(),
        kind: BuildingKind::TownCenter,
        position: Position { x, y },
        produces: vec![ProductKind::Villager],
        researches: TechnologyKind::ALL.to_vec(),
        job: None,
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

fn compatible_biomes(kind: ResourceKind) -> &'static [TerrainBiome] {
    match kind {
        ResourceKind::Wood => &[TerrainBiome::Forest, TerrainBiome::Heath],
        ResourceKind::Food => &[TerrainBiome::Meadow, TerrainBiome::Prairie],
        ResourceKind::Stone => &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        ResourceKind::Gold => &[TerrainBiome::Highland],
        ResourceKind::Iron => &[TerrainBiome::Highland, TerrainBiome::Scrubland],
        ResourceKind::Clay => &[TerrainBiome::Clayland, TerrainBiome::Wetland],
        ResourceKind::Fiber => &[TerrainBiome::Wetland, TerrainBiome::Prairie],
        ResourceKind::Coal
        | ResourceKind::Timber
        | ResourceKind::Steel
        | ResourceKind::Bricks
        | ResourceKind::Cloth
        | ResourceKind::Rations => &[],
    }
}

fn generate_resources(terrain: &[TerrainCell]) -> Vec<ResourceNode> {
    const SPECS: [(ResourceKind, &str, usize, f64); 7] = [
        (ResourceKind::Wood, "tree", 6, 25.0),
        (ResourceKind::Food, "berries", 4, 50.0),
        (ResourceKind::Stone, "stone", 4, 40.0),
        (ResourceKind::Gold, "gold", 2, 35.0),
        (ResourceKind::Iron, "iron", 2, 40.0),
        (ResourceKind::Clay, "clay", 2, 45.0),
        (ResourceKind::Fiber, "fiber", 2, 50.0),
    ];

    let mut resources: Vec<ResourceNode> = Vec::new();
    for (kind, prefix, count, amount) in SPECS {
        for number in 1..=count {
            let cell = terrain
                .iter()
                .filter(|cell| compatible_biomes(kind).contains(&cell.biome))
                .filter(|cell| {
                    let position = Position {
                        x: (f64::from(cell.column) + 0.5) * CELL_SIZE,
                        y: (f64::from(cell.row) + 0.5) * CELL_SIZE,
                    };
                    position.distance(Position {
                        x: 1200.0,
                        y: 800.0,
                    }) >= STARTING_BASE_RESOURCE_CLEARANCE
                        && resources.iter().all(|resource| {
                            resource.position.distance(position) + f64::EPSILON
                                >= RESOURCE_MIN_SEPARATION
                        })
                })
                .min_by_key(|cell| {
                    let dx = i32::from(cell.column) - i32::from(WORLD_COLUMNS / 2);
                    let dy = i32::from(cell.row) - i32::from(WORLD_ROWS / 2);
                    (dx * dx + dy * dy, cell.row, cell.column)
                })
                .expect("fixed terrain has enough separated biome-compatible resource cells");
            resources.push(ResourceNode {
                id: format!("{prefix}-{number}"),
                kind,
                position: Position {
                    x: (f64::from(cell.column) + 0.5) * CELL_SIZE,
                    y: (f64::from(cell.row) + 0.5) * CELL_SIZE,
                },
                amount,
            });
        }
    }
    resources
}

impl GameWorld {
    pub fn apply_command(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::Move { unit_id, x, y } => {
                let unit_index = self.unit_index_and_idle(&unit_id)?;
                if !self.contains_position(x, y) {
                    return Err(CommandError::InvalidDestination);
                }
                let target = Position { x, y };
                self.route_exists_to_free_cell(unit_index, target)?;
                self.units[unit_index].action = UnitAction::Move { x, y };
                Ok(())
            }
            Command::GroupMove { unit_ids, x, y } => {
                if !self.contains_position(x, y) {
                    return Err(CommandError::InvalidDestination);
                }
                if unit_ids.is_empty() {
                    return Err(CommandError::EmptyUnitGroup);
                }
                let mut seen = BTreeSet::new();
                let mut members = Vec::with_capacity(unit_ids.len());
                for unit_id in unit_ids {
                    if !seen.insert(unit_id.clone()) {
                        return Err(CommandError::DuplicateUnit);
                    }
                    members.push((unit_id.clone(), self.unit_index_and_idle(&unit_id)?));
                }
                members.sort_by(|left, right| left.0.cmp(&right.0));
                let member_indices: Vec<_> = members.iter().map(|(_, index)| *index).collect();
                let assignments =
                    self.group_move_assignments(&member_indices, Position { x, y })?;
                for (unit_index, destination) in assignments {
                    self.units[unit_index].action = UnitAction::Move {
                        x: destination.x,
                        y: destination.y,
                    };
                }
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
                if !self.interaction_route_exists(unit_index, resource.position) {
                    return Err(CommandError::TargetUnreachable);
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
                let site = Position { x, y };
                if self.site_is_occupied(site) {
                    return Err(CommandError::InvalidBuildSite);
                }
                if !self.interaction_route_exists(unit_index, site) {
                    return Err(CommandError::TargetUnreachable);
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
                let building_index = self.building_index_and_idle(&building_id)?;
                if !self.buildings[building_index].produces.contains(&product) {
                    return Err(CommandError::ProductUnavailable);
                }
                match product {
                    ProductKind::Villager if self.stockpile.food < VILLAGER_FOOD_COST => {
                        return Err(CommandError::InsufficientFood);
                    }
                    ProductKind::Villager => self.stockpile.food -= VILLAGER_FOOD_COST,
                }
                self.buildings[building_index].job = Some(BuildingJob::Produce {
                    product,
                    elapsed_seconds: 0.0,
                });
                Ok(())
            }
            Command::Research {
                building_id,
                technology,
            } => {
                let building_index = self.building_index_and_idle(&building_id)?;
                if !self.buildings[building_index]
                    .researches
                    .contains(&technology)
                {
                    return Err(CommandError::TechnologyUnavailable);
                }
                if self.researched_technologies.contains(&technology) {
                    return Err(CommandError::TechnologyAlreadyResearched);
                }
                if technology
                    .prerequisite()
                    .is_some_and(|required| !self.researched_technologies.contains(&required))
                {
                    return Err(CommandError::MissingTechnologyPrerequisite);
                }
                if self.stockpile.food < RESEARCH_FOOD_COST
                    || self.stockpile.wood < RESEARCH_WOOD_COST
                {
                    return Err(CommandError::InsufficientResearchResources);
                }
                self.stockpile.food -= RESEARCH_FOOD_COST;
                self.stockpile.wood -= RESEARCH_WOOD_COST;
                self.buildings[building_index].job = Some(BuildingJob::Research {
                    technology,
                    elapsed_seconds: 0.0,
                });
                Ok(())
            }
            Command::SetSimulationSpeed { multiplier } => {
                if ![0.0, 1.0, 2.0].contains(&multiplier) {
                    return Err(CommandError::InvalidSimulationSpeed);
                }
                self.simulation_speed = multiplier;
                Ok(())
            }
        }
    }

    fn building_index_and_idle(&self, building_id: &str) -> Result<usize, CommandError> {
        let index = self
            .buildings
            .iter()
            .position(|building| building.id == building_id)
            .ok_or(CommandError::BuildingNotFound)?;
        if self.buildings[index].job.is_some() {
            return Err(CommandError::BuildingBusy);
        }
        Ok(index)
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
        let dt = dt * self.simulation_speed;
        if dt <= 0.0 {
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
            self.tick_building_job(index, dt);
        }
        self.refresh_exploration();
        // Slice A records terminal scenario state without enforcing Slice E's
        // command restrictions. Gameplay keeps simulating, but terminal scenario
        // progress is immutable once won or lost.
        if self.scenario.outcome == ScenarioOutcome::Running {
            self.scenario.elapsed_ticks = self.scenario.elapsed_ticks.saturating_add(1);
            if self.scenario.elapsed_ticks >= self.scenario.tick_limit {
                self.scenario.outcome = ScenarioOutcome::Lost;
            }
        }
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
            simulation_speed: self.simulation_speed,
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
            researched_technologies: self.researched_technologies.clone(),
            catalog: DomainCatalog::roadmap(),
            scenario: self.scenario.clone(),
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

    pub(super) fn cell_for_position(&self, position: Position) -> Option<CellCoordinate> {
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

    pub(super) fn gather_multiplier(&self, resource: ResourceKind) -> f64 {
        if self
            .researched_technologies
            .iter()
            .any(|technology| technology.improves(resource))
        {
            GATHERING_TECH_MULTIPLIER
        } else {
            1.0
        }
    }

    fn tick_build(&mut self, unit_index: usize, target: Position, work_seconds: f64, dt: f64) {
        let (arrived, remaining) = self.tick_toward_interaction(unit_index, target, dt);
        if !arrived {
            return;
        }
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

    fn tick_building_job(&mut self, building_index: usize, dt: f64) {
        let Some(job) = self.buildings[building_index].job.clone() else {
            return;
        };
        match job {
            BuildingJob::Produce {
                product,
                mut elapsed_seconds,
            } => {
                elapsed_seconds += dt;
                if elapsed_seconds + f64::EPSILON < VILLAGER_PRODUCTION_SECONDS {
                    self.buildings[building_index].job = Some(BuildingJob::Produce {
                        product,
                        elapsed_seconds,
                    });
                    return;
                }
                match product {
                    ProductKind::Villager => {
                        let Some(position) = self.spawn_position(building_index) else {
                            self.buildings[building_index].job = Some(BuildingJob::Produce {
                                product,
                                elapsed_seconds,
                            });
                            return;
                        };
                        self.units.push(Unit {
                            id: format!("villager-{}", self.next_unit_id),
                            kind: UnitKind::Villager,
                            position,
                            action: UnitAction::Idle,
                            cargo: None,
                        });
                        self.next_unit_id += 1;
                    }
                }
            }
            BuildingJob::Research {
                technology,
                mut elapsed_seconds,
            } => {
                elapsed_seconds += dt;
                if elapsed_seconds + f64::EPSILON < RESEARCH_SECONDS {
                    self.buildings[building_index].job = Some(BuildingJob::Research {
                        technology,
                        elapsed_seconds,
                    });
                    return;
                }
                self.researched_technologies.push(technology);
                self.researched_technologies.sort_unstable();
            }
        }
        self.buildings[building_index].job = None;
    }
}

pub(super) fn move_along(
    position: &mut Position,
    waypoints: impl IntoIterator<Item = Position>,
    mut dt: f64,
) -> f64 {
    for target in waypoints {
        dt = move_toward(position, target, dt);
        if *position != target || dt <= f64::EPSILON {
            return 0.0;
        }
    }
    dt
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
