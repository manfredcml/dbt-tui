//! UI Components
//!
//! Each component encapsulates its own state, event handling, and rendering logic.
//! Components communicate through Actions rather than direct state mutation.

pub mod detail;
pub mod documentation;
pub mod help_dialog;
pub mod history_dialog;
pub mod home;
pub mod info;
pub mod layout;
pub mod lineage;
pub mod quit_dialog;
pub mod run_options_dialog;
pub mod run_output_dialog;
pub mod sample_data_dialog;
pub mod setup;
pub mod splash;
pub mod sql_highlight;
pub mod table;
pub mod tag_filter_dialog;
pub mod target_selector;

pub use detail::DetailComponent;
pub use documentation::DocumentationComponent;
pub use help_dialog::HelpDialog;
pub use history_dialog::HistoryDialog;
pub use home::{draw_home_screen, HomeComponent, HomeRenderContext};
pub use info::ProjectInfoDialog;
pub use layout::{calculate_main_layout, centered_popup};
pub use lineage::LineageComponent;
pub use quit_dialog::QuitDialog;
pub use run_options_dialog::RunOptionsDialog;
pub use run_output_dialog::RunOutputDialog;
pub use sample_data_dialog::SampleDataDialog;
pub use setup::SetupComponent;
pub use splash::SplashComponent;
pub use table::TableComponent;
pub use tag_filter_dialog::TagFilterDialog;
pub use target_selector::TargetSelectorDialog;
