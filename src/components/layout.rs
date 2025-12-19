//! Layout calculations for the UI

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main screen layout areas
pub struct MainLayout {
    pub tabs: Rect,
    pub info: Rect,
    pub list: Rect,
    pub detail: Rect,
    pub lineage: Option<Rect>,
    pub documentation: Option<Rect>,
    pub status: Option<Rect>,
    pub help: Rect,
}

/// Calculate centered popup area
pub fn centered_popup(area: Rect, width: u16, height: u16) -> Rect {
    let popup_x = (area.width.saturating_sub(width)) / 2;
    let popup_y = (area.height.saturating_sub(height)) / 2;

    Rect::new(
        popup_x,
        popup_y,
        width.min(area.width),
        height.min(area.height),
    )
}

/// Calculate main screen layout
pub fn calculate_main_layout(
    area: Rect,
    has_status: bool,
    show_lineage: bool,
    show_documentation: bool,
) -> MainLayout {
    // Main vertical layout: content + (optional status) + help bar
    let main_chunks = if has_status {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area)
    };

    // Horizontal split: left panel (20%) and right panel (80%)
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(main_chunks[0]);

    // Left panel: tabs + info box + list
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(0),
        ])
        .split(horizontal_chunks[0]);

    // Right panel layout depends on lineage/documentation visibility
    let (detail_area, lineage_area, documentation_area) = if show_lineage && show_documentation {
        let right_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(horizontal_chunks[1]);
        (right_chunks[0], Some(right_chunks[1]), Some(right_chunks[2]))
    } else if show_lineage {
        let right_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(horizontal_chunks[1]);
        (right_chunks[0], Some(right_chunks[1]), None)
    } else if show_documentation {
        let right_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(horizontal_chunks[1]);
        (right_chunks[0], None, Some(right_chunks[1]))
    } else {
        (horizontal_chunks[1], None, None)
    };

    let (status_area, help_area) = if has_status {
        (Some(main_chunks[1]), main_chunks[2])
    } else {
        (None, main_chunks[1])
    };

    MainLayout {
        tabs: left_chunks[0],
        info: left_chunks[1],
        list: left_chunks[2],
        detail: detail_area,
        lineage: lineage_area,
        documentation: documentation_area,
        status: status_area,
        help: help_area,
    }
}
