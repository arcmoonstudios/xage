// src/omnixtracker/mod.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[OMNIXTRACKER]Xyn>=====S===t===u===d===i===o===s======[R|$>

pub mod omnixerror;
pub mod omnixmetry;

pub use omnixerror::{
    OmniXError, OmniXErrorManager, OmniXErrorManagerConfig, handle_build_error, handle_main_error,
};
pub use omnixmetry::{OmniXMetry, setup_global_subscriber};