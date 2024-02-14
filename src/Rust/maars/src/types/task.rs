use crate::{bind, Direction};
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use std::{fmt::Display, path::PathBuf};

/// Represents the ID of a task.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Serialize)]
#[repr(transparent)]
pub struct TaskId(pub(crate) bind::AsstTaskId);

impl Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TaskId {
    /// Creates a new [`TaskId`]
    ///
    /// # Parameters
    /// * `id`:  The numerical id of the task
    pub fn new(id: i32) -> Self {
        Self(id)
    }
}

// #[derive(Debug, Clone)]
// pub struct Task {
//     pub id: TaskId,

//     /// The type of the task
//     /// TODO: Convert the task type to a Rust enum
//     pub task_type: String,

//     /// The parameters for the task, serialized as a JSON string
//     pub params: String,
// }

#[derive(Debug, Clone, Serialize, strum::Display, EnumDiscriminants)]
#[serde(untagged)]
pub enum Task {
    /// Starts the game
    StartUp {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// The client that is used
        #[serde(rename = "client_type")]
        client: Option<ClientType>,

        /// Whether to lauch the client when this task runs
        #[serde(rename = "start_game_enabled", default)]
        start_game: bool,
    },

    /// Closes the game client
    CloseDown {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,
    },

    /// Runs an operation
    ///
    /// Supports some of the special stages. Please refer to the autoLocalization in the
    /// MAA core repository.
    Fight {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// The name of the stage to run.
        ///
        /// If this is not set, then the last run operation
        /// will be run by default. Supports all mainline stages, such as "1-7", "S3-2",
        /// etc. At the end of the level, enter Normal/Hard to switch between Normal and
        /// Tough difficulty Annihilation. The input should be `Annihilation`. Certain
        /// side story stages. The input should be complete with stage number.
        ///
        /// Editing in run-time is not supported!
        stage: Option<String>,

        /// The number of sanity potions to use. If not set, no potions will be used.
        #[serde(rename = "medicine")]
        sanity_pots: Option<u32>,

        /// The number of expiring sanity potions to use. If not set, no potions will be
        /// used.
        #[serde(rename = "expiring_medicine")]
        expiring_sanity_pots: Option<String>,

        /// The number of orginite prime to use. If not set, no orginite prime will be
        /// used.
        #[serde(rename = "stone")]
        orginite_prime: Option<u32>,

        /// The number of times to run the stage. If not set, the stage will be run
        /// infinitely.
        times: Option<u32>,

        /// Specifying the number of drops. Items are combined with a boolean OR, i.e. the
        /// task ends when any of the items was collected the specified number of times.
        drops: Option<Vec<(ItemId, u32)>>,

        /// Whether to report item drops to PenguinStats.
        #[serde(default)]
        report_to_penguin: bool,

        /// The PenguinStats id token. Only used if the [`report`] field is set to true.
        penguin_id: Option<String>,

        /// The server to report the item drops for.
        #[serde(default)]
        server: Option<StatsServer>,

        /// Client version.
        ///
        /// This field is used to reconnect after the game crashed. [`None`] means to
        /// disable this feature.
        client: Option<ClientType>,

        /// Whether to save sanity by using Originite Prime.
        ///
        /// This setting is only in e effect when Originite Prime may be used. It waits on
        /// the sanity confirmation screen until 1 point of sanity has been restored and
        /// then immediately use Originite Prime.
        #[serde(rename = "DrGrandet", default)]
        dr_grandet: bool,
    },

    /// Task to recruit an operator
    Recruitment {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// Whether to refresh 3★ tags
        #[serde(default)]
        refresh: bool,

        /// The operator levels to recruit
        /// TODO: This should be a bitflags enum, probably.
        #[serde(rename = "select")]
        operator_levels: Vec<u32>,

        /// The tag levels that require user confirmation to proceed
        #[serde(rename = "confirm")]
        confirm_levels: Vec<u32>,

        /// The mode to select tags
        #[serde(rename = "extra_tags_mode", default)]
        tags_mode: TagsMode,

        /// How often the recruitment should be run
        #[serde(rename = "times", default)]
        repretitions: u32,

        /// Whether to set the recruitment time to nine hours. jthis settings is only
        /// available when the [´repetitions`] field is set to 0. Defaults to true.
        #[serde(default = "default_true")]
        set_time: bool,

        /// Whether to expedite the recruitment with plans
        #[serde(default)]
        expedite: bool,

        /// The number of expedite plans to use
        #[serde(rename = "expedire_times", default)]
        expedite_times: u32,

        /// Whether to skip when a robot tag is present
        #[serde(default = "default_true")]
        skip_robot: bool,

        /// How long the recruitment for a givenn operator rarity should take
        /// TODO: This could be an actual struct, since the operator rarity is a fixed set
        recruitment_times: Vec<(String, i32)>,

        /// Whether to report recruitments to PenguinStats.
        #[serde(default)]
        report_to_penguin: bool,

        /// The PenguinStats id token. Only used if the [`report`] field is set to true.
        penguin_id: Option<String>,

        /// Whether to report recruitments to YITULIU.
        #[serde(default)]
        report_to_yituliu: bool,

        /// The YITULIU id token. Only used if the [`report`] field is set to true.
        yituliu_id: Option<String>,

        /// The server to report the recruitments for.
        #[serde(default)]
        server: Option<StatsServer>,
    },

    /// Task to manage the base and op  erators in it
    #[serde(rename = "Infrast")]
    Infrastructure {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// How oprataors are shifted around in the base
        ///
        /// Editing in run-time is not supported.
        ///
        /// # Possible Values
        /// * `0`: Default behavior
        /// * `10000`: Custom Mode, please refer 3.6-INFRASTRUCTURE_SCHEDULING_SCHEMA
        shift: ShiftMode,

        /// The facilities where opoerators can be shifte in or out
        ///
        /// Editing in run-time is not supported.
        #[serde(rename = "facility")]
        facilities: Vec<Facility>,

        /// Whether to use drones to speed up production
        #[serde(rename = "dones", default)]
        use_drones: bool,

        /// The threshols at which operators are shifted out of a facility.
        #[serde(default = "default_morale")]
        threshold: f32,

        ///  Whether to replenish Originium Shards
        #[serde(rename = "replenish", default)]
        replenish_shards: bool,

        /// Whether to enbale "Not Stationed in Dorm"
        dorm_notstationed_enabled: bool,

        /// Whether to fill operators in dorm by trust
        dorm_trust_enabled: bool,
    },

    /// Task to collect credits and auto-purchase items
    ///
    /// Purchase happens in order by following the `buy_first` list. After that any other
    /// items are bought from left to right ignoring items in the `blacklist`. If credits
    /// are overflowing, a third pass is made, bying  items from left to right, ignoring
    /// the `blacklist`.
    Mall {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// Whether to buy items from the store
        #[serde(default)]
        shopping: bool,

        /// The items to buy first
        #[serde(default)]
        buy_first: Vec<String>,

        /// List of items that should not be bought.
        ///
        /// This field is ignored when the credits are overflowing and
        /// [`force_shopping_if_credit_full`]. Editing in run-time is not supported.
        #[serde(default)]
        blacklist: Vec<String>,

        /// Whether to force shopping if credits are full. The blacklist is ignored in
        /// this case.
        #[serde(default = "default_true")]
        force_shopping_if_credit_full: bool,

        /// Whether to purchase only discounted items, applicable only on the second round
        /// of purchases.
        #[serde(default)]
        only_buy_discount: bool,

        ///  Whether to stop purchasing when credit points fall below 300, applicable only
        ///  on the second round of purchases
        #[serde(default)]
        reserve_max_credit: bool,
    },

    /// Task to collects the daily rewards
    Award {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,
    },

    /// Task to fight in Integrated Strategies (IS) stages
    Roguelike {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// The version of IS to use
        theme: IntegratedStrategiesTheme,

        /// How the them should be played
        mode: ExitMode,

        /// How often IS should be run
        #[serde(rename = "starts_count", default = "max_amount")]
        num_starts: i32,

        /// Whether to enable investments
        investment_enabled: bool,

        /// The number of investments to make
        #[serde(rename = "investments_count", default = "max_amount")]
        num_investments: i32,

        /// Stop when investments are reached
        #[serde(default)]
        stop_when_investment_full: bool,

        /// The name of the squad to use, e.g. "assault squad"
        squad: Option<String>,

        /// The roles
        ///
        /// TODO: Improve documentation
        roles: Option<String>,

        /// The core operator to use. Recognizes auto-selection of levels.
        #[serde(rename = "core_char")]
        core_operator: Option<String>,

        /// Whether the core operator can be a support unit
        #[serde(default)]
        use_support: bool,

        /// Whether to use a support unit from players that are not on the firends list
        #[serde(default)]
        use_nonfriend_support: bool,

        /// Whether refresh trader with dice to buy special items
        #[serde(rename = "refresh_trader_with_dice", default)]
        refresh_trader: bool,
    },

    /// Task to run the copilot auto  combat feature
    ///
    /// For more information, please refer to chapter 3.6 _COPILOT_SCHEMA_ in the MAA
    /// repository.
    Copilot {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// The full path to the task JSON file.
        ///
        /// Editing in run-time is not supported.
        filname: PathBuf,

        /// Whether to "quick build"
        ///
        /// Editing in run-time is not supported.
        #[serde(default)]
        formation: bool,
    },

    /// Task to run the copilot auto combat feature for _STATIONARY SECURITY SERVICE_
    /// (SSS)
    SSSCopilot {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// The full path to the task JSON file.
        ///
        /// Editing in run-time is not supported.
        filname: PathBuf,

        /// How often to repeat
        #[serde(rename = "loop_times", default)]
        num_loops: bool,
    },

    /// Task to auto recognize items in the players depot
    #[serde(rename = "Depot")]
    #[strum(serialize = "Depot")]
    DepotRecognition {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,
    },

    /// Task to recognize all operators a player has
    #[serde(rename = "OperBox")]
    #[strum(serialize = "OperBox")]
    OperatorRecognition {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,
    },

    /// Task to run the reclamation algorithm mode
    ReclamationAlgorithm {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// The theme to play
        theme: ReclamationAlgorithmTheme,

        /// The mode to play
        mode: ReclamationAlgorithmMode,
    },

    /// Task to run a custom task list
    Custom {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// Execute the first task the list
        ///
        /// If multiple should be performed, then this task should be added multiple times
        /// to the task list.
        task_names: Vec<String>,
    },

    /// Task to take a single step in the game
    SingleStep {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// The type of step to take
        step_type: StepType,

        /// The subtask
        subtask: Subtask,
    },

    /// Task to use a video to generate a list of tasks to automate playing a mission
    VideoRecognition {
        /// Whether to enable this task, optional inm JSON, true by default
        #[serde(default = "default_true", rename = "enable")]
        enabled: bool,

        /// Path to the video file
        filename: PathBuf,
    },
}

/// Enumerates the possible clients that in which Arknights can be played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, Serialize)]
pub enum ClientType {
    Official,
    Bilibili,
    #[strum(serialize = "txwy")]
    Txwy,

    #[strum(serialize = "YostarEN")]
    #[serde(rename = "YostarEN")]
    EN,

    #[strum(serialize = "YostarJP")]
    #[serde(rename = "YostarJP")]
    JP,

    #[strum(serialize = "YostarKR")]
    #[serde(rename = "YostarKR")]
    KR,
}

fn default_true() -> bool {
    true
}

/// The nummerical ID of an item
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Serialize)]
pub struct ItemId(pub(crate) u32);

impl Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ItemId {
    /// Creates a new [`ItemId`]
    ///
    /// # Parameters
    /// * `id`:  The numerical id of the item
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Enumerates the possible PenguinStats servers to report item drops to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, Serialize, Default)]
pub enum StatsServer {
    #[default]
    CN,
    US,
    JP,
    KR,
}

/// Enumerates how many tags should be selected when recruiting an operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display, Serialize, Default)]
pub enum TagsMode {
    /// No additional tags are selected
    #[default]
    Default = 0,

    /// The selection will always have three tags, even if the tags are conflicting with
    /// each other.
    AlwaysThree,

    /// The selection will pick as many high rarity tags as possible, even if the tags are
    /// conflicting with each other.
    MaximumPossible,
}

/// Enumerates the possible ways to shift operators in the base when their morale reaches
/// the lower bound.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default, strum::Display)]
pub enum ShiftMode {
    /// Use the defaultshifting behavoiur
    #[default]
    Default,

    /// Use the custom shifting behaviour.
    ///
    /// Please refer to the 3.6-INFRASTRUCTURE_SCHEDULING_SCHEMA in the MAA respository
    /// for more information.
    /// TODO: Improve documentation
    Custom {
        /// The path to the file containing the custom schedules
        filename: PathBuf,

        /// The index of the schedule to use
        index: usize,
    },
}

/// Enumerates the possible facilities in the base
#[derive(Debug, Clone, PartialEq, Eq, Serialize, strum::Display)]
pub enum Facility {
    #[serde(rename = "Mfg")]
    #[strum(serialize = "Mfg")]
    Factory,

    #[serde(rename = "Trade")]
    #[strum(serialize = "Trade")]
    TradingPost,

    #[serde(rename = "Power")]
    #[strum(serialize = "Power")]
    PowerPlant,

    #[serde(rename = "Control")]
    #[strum(serialize = "Control")]
    ControlCenter,

    Reception,
    Office,
    Dorm,
}

fn default_morale() -> f32 {
    0.3
}

fn max_amount() -> i32 {
    i32::MAX
}

/// Enumerates the possible themes for Integrated Strategies
#[derive(Debug, Clone, PartialEq, Eq, Serialize, strum::Display, Default)]
pub enum IntegratedStrategiesTheme {
    #[default]
    Phantom,
    Mizuki,
}

/// Enumerates the possible exit modes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, strum::Display, Default)]
pub enum ExitMode {
    /// Play as much as possible with a stable strategy
    #[default]
    Candle = 0,

    /// Exit after the first level
    OriginiumIngots,

    /// Play until invests
    Both,

    /// Play as much as possible with an aggressive strategy
    Pass,
}

/// Enumerates the possible reclamation algorithm themes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, strum::Display, Default)]
pub enum ReclamationAlgorithmTheme {
    #[default]
    #[strum(serialize = "Fire Within the Sand")]
    FireWithinSand = 0,

    #[strum(serialize = "Tales Within the Sand")]
    TalesWithinSand,
}

/// Enumerates the possible reclamation algorithm modes.
///
/// Curretly only [`FireWithinSand`](ReclamationAlgorithmTheme::FireWithinSand) supports this field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, strum::Display, Default)]
pub enum ReclamationAlgorithmMode {
    /// Farm badges & construction pts and then exits the stage immediately
    #[default]
    FarmBadges = 0,

    /// Farm Crude Gold anf forge it into Gold at headquarter after purchasing water
    FarmCrudeGold,
}

/// Enumerates the possible types a single step task can have
#[derive(Debug, Clone, PartialEq, Eq, Serialize, strum::Display)]
pub enum StepType {
    Copilot,
}

/// Enumerates the possible subtasks for a single step task
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Subtask {
    Stage {
        stage: String,
    },
    Start,

    Action {
        name: String,
        location: (isize, isize),
        direction: Direction,
    },
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_name() {
        let data = Task::DepotRecognition {
            enabled: true,
        };

        let name = data.to_string();
        dbg!(name);

        let json = serde_json::to_string(&data).unwrap();
        dbg!(json);
    }
}
