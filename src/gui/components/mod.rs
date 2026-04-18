//! UI components for the MALD GUI.
//!
//! These are reusable view components that compose to form the main layout:
//! - `activity_bar`: 48px vertical icon strip on the left
//! - `sidebar_content`: Mode-dependent sidebar content
//! - `top_search`: Centered search bar at top of editor area
//! - `tab_bar`: Styled editor tab bar
//! - `feature_panel`: Right panel for backlinks, AI chat, outline
//! - `terminal_panel`: Bottom terminal with styled header

pub mod activity_bar;
pub mod feature_panel;
pub mod sidebar_content;
pub mod tab_bar;
pub mod terminal_panel;
pub mod top_search;
