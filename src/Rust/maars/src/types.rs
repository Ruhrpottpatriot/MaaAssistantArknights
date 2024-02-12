//! Holds the definitions for the types used in the crate. While all doc-comments try to
//! faithfully represent and expand on the documentation in the original C/C++ code, it is
//! nonetheless a copy. In case the two contradict, the [original comments](https://github.com/MaaAssistantArknights/MaaAssistantArknights/blob/dev/src/MaaCore/Common/AsstTypes.h) take precedence.

use strum::{Display, EnumString};

/// Enumerates the possible process-wide options for the assistant
///
/// Process-wide options are set during startup and apply to all instances of the
/// assistant. They cannot be changed after startup and changes need a process restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticOption {
    /// The CPU is used during OCR
    CpuOCR,

    /// The GPU is used during OCR
    GpuOCR { gpu_id: usize },
}

/// Enumerates the possible instance-wide options for the assistant
///
/// Instance wide options are set during the cration of an [`Assistant`] and and apply to
/// that instance only. They can be changed at any time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceOption {
    #[deprecated(note = "Use `TouchMode` instead")]
    MinitouchEnabled(bool),

    /// The touch mode to use
    TouchMode(TouchMode),

    /// Whether to pause the deployment after each click
    DeploymentWithPause(bool),

    /// Whether to use ADB Lite
    AdbLiteEnabled(bool),

    /// Whether to kill ADB on exit
    KillAdbOnExit(bool),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum TouchMode {
    /// Use ADB for touch input
    #[default]
    #[strum(serialize = "adb")]
    ADB,

    /// Use MiniTouch for touch input
    #[strum(serialize = "minitouch")]
    Minitouch,

    #[strum(serialize = "maatouch", serialize = "MAATouch")]
    MaaTouch,

    #[strum(serialize = "MacPlayTools")]
    MacPlayTools,
}
