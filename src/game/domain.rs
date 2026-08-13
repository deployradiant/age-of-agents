use serde::{Deserialize, Serialize};

pub const DEFAULT_SCENARIO_TICK_LIMIT: u64 = 36_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub(super) fn distance(self, other: Self) -> f64 {
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
    pub(super) fn coordinate(self) -> CellCoordinate {
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
    #[serde(default)]
    pub kind: UnitKind,
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
    Gold,
    Iron,
    Coal,
    Clay,
    Fiber,
    Timber,
    Steel,
    Bricks,
    Cloth,
    Rations,
}

impl ResourceKind {
    pub const ALL: [Self; 13] = [
        Self::Wood,
        Self::Food,
        Self::Stone,
        Self::Gold,
        Self::Iron,
        Self::Coal,
        Self::Clay,
        Self::Fiber,
        Self::Timber,
        Self::Steel,
        Self::Bricks,
        Self::Cloth,
        Self::Rations,
    ];
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitKind {
    #[default]
    Villager,
    Guard,
    Archer,
    Healer,
    SiegeCart,
}

impl UnitKind {
    pub const ALL: [Self; 5] = [
        Self::Villager,
        Self::Guard,
        Self::Archer,
        Self::Healer,
        Self::SiegeCart,
    ];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub kind: BuildingKind,
    pub position: Position,
    pub produces: Vec<ProductKind>,
    pub researches: Vec<TechnologyKind>,
    pub job: Option<BuildingJob>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingKind {
    TownCenter,
    MiningCamp,
    Farm,
    LumberMill,
    Smelter,
    Kiln,
    Weaver,
    Kitchen,
    Barracks,
    Range,
    Workshop,
    Infirmary,
    Watchtower,
    Monument,
}

impl BuildingKind {
    pub const ALL: [Self; 14] = [
        Self::TownCenter,
        Self::MiningCamp,
        Self::Farm,
        Self::LumberMill,
        Self::Smelter,
        Self::Kiln,
        Self::Weaver,
        Self::Kitchen,
        Self::Barracks,
        Self::Range,
        Self::Workshop,
        Self::Infirmary,
        Self::Watchtower,
        Self::Monument,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductKind {
    Villager,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BuildingJob {
    Produce {
        product: ProductKind,
        elapsed_seconds: f64,
    },
    Research {
        technology: TechnologyKind,
        elapsed_seconds: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnologyKind {
    Forestry,
    Agriculture,
    Masonry,
    Mining,
    Textiles,
}

impl TechnologyKind {
    pub const ALL: [Self; 5] = [
        Self::Forestry,
        Self::Agriculture,
        Self::Masonry,
        Self::Mining,
        Self::Textiles,
    ];

    pub(super) fn prerequisite(self) -> Option<Self> {
        match self {
            Self::Mining => Some(Self::Masonry),
            Self::Textiles => Some(Self::Agriculture),
            Self::Forestry | Self::Agriculture | Self::Masonry => None,
        }
    }

    pub(super) fn improves(self, resource: ResourceKind) -> bool {
        matches!(
            (self, resource),
            (Self::Forestry, ResourceKind::Wood)
                | (Self::Agriculture, ResourceKind::Food)
                | (Self::Masonry, ResourceKind::Stone | ResourceKind::Clay)
                | (Self::Mining, ResourceKind::Gold | ResourceKind::Iron)
                | (Self::Textiles, ResourceKind::Fiber)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stockpile {
    pub wood: f64,
    pub food: f64,
    pub stone: f64,
    pub gold: f64,
    pub iron: f64,
    #[serde(default)]
    pub coal: f64,
    pub clay: f64,
    pub fiber: f64,
    #[serde(default)]
    pub timber: f64,
    #[serde(default)]
    pub steel: f64,
    #[serde(default)]
    pub bricks: f64,
    #[serde(default)]
    pub cloth: f64,
    #[serde(default)]
    pub rations: f64,
}

impl Stockpile {
    pub(super) fn add(&mut self, kind: ResourceKind, amount: f64) {
        match kind {
            ResourceKind::Wood => self.wood += amount,
            ResourceKind::Food => self.food += amount,
            ResourceKind::Stone => self.stone += amount,
            ResourceKind::Gold => self.gold += amount,
            ResourceKind::Iron => self.iron += amount,
            ResourceKind::Coal => self.coal += amount,
            ResourceKind::Clay => self.clay += amount,
            ResourceKind::Fiber => self.fiber += amount,
            ResourceKind::Timber => self.timber += amount,
            ResourceKind::Steel => self.steel += amount,
            ResourceKind::Bricks => self.bricks += amount,
            ResourceKind::Cloth => self.cloth += amount,
            ResourceKind::Rations => self.rations += amount,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeKind {
    Timber,
    Steel,
    Bricks,
    Cloth,
    Rations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeCatalogEntry {
    pub kind: RecipeKind,
    pub building: BuildingKind,
    pub inputs: Vec<ResourceKind>,
    pub output: ResourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainCatalog {
    pub resources: Vec<ResourceKind>,
    pub buildings: Vec<BuildingKind>,
    pub units: Vec<UnitKind>,
    pub recipes: Vec<RecipeCatalogEntry>,
    pub technologies: Vec<TechnologyKind>,
}

impl DomainCatalog {
    pub fn roadmap() -> Self {
        use BuildingKind::{Kiln, Kitchen, LumberMill, Smelter, Weaver};
        use RecipeKind::{Bricks, Cloth, Rations, Steel, Timber};
        use ResourceKind::{Clay, Coal, Fiber, Food, Iron, Wood};

        Self {
            resources: ResourceKind::ALL.to_vec(),
            buildings: BuildingKind::ALL.to_vec(),
            units: UnitKind::ALL.to_vec(),
            recipes: vec![
                recipe(Timber, LumberMill, &[Wood], ResourceKind::Timber),
                recipe(Steel, Smelter, &[Iron, Coal], ResourceKind::Steel),
                recipe(Bricks, Kiln, &[Clay, Wood], ResourceKind::Bricks),
                recipe(Cloth, Weaver, &[Fiber], ResourceKind::Cloth),
                recipe(Rations, Kitchen, &[Food], ResourceKind::Rations),
            ],
            technologies: TechnologyKind::ALL.to_vec(),
        }
    }
}

fn recipe(
    kind: RecipeKind,
    building: BuildingKind,
    inputs: &[ResourceKind],
    output: ResourceKind,
) -> RecipeCatalogEntry {
    RecipeCatalogEntry {
        kind,
        building,
        inputs: inputs.to_vec(),
        output,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioId {
    Prototype,
    FoundryTown,
    FrontierSurvey,
    MonumentWorks,
    HoldTheCoast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioOutcome {
    Running,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioObjectiveProgress {
    pub completed: u16,
    pub total: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioState {
    pub id: ScenarioId,
    pub tick_limit: u64,
    #[serde(default)]
    pub elapsed_ticks: u64,
    pub objective_progress: ScenarioObjectiveProgress,
    pub outcome: ScenarioOutcome,
}

impl Default for ScenarioState {
    fn default() -> Self {
        Self {
            id: ScenarioId::Prototype,
            tick_limit: DEFAULT_SCENARIO_TICK_LIMIT,
            elapsed_ticks: 0,
            objective_progress: ScenarioObjectiveProgress {
                completed: 0,
                total: 0,
            },
            outcome: ScenarioOutcome::Running,
        }
    }
}
