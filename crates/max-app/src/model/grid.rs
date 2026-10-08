use max_api::cms::Layout;

use super::rail::{ScreenRow, ScreenTile};
use crate::metrics::{self, rail::TILE_GAP, MARGIN};

pub fn tiles_across(layout: Layout, width: f32) -> usize {
    let tile = metrics::tile_size(layout).x;
    ((width - MARGIN * 2.0 + TILE_GAP) / (tile + TILE_GAP)).floor().max(1.0) as usize
}

pub fn grid_lines(mut tiles: Vec<ScreenTile>, layout: Layout, width: f32) -> Vec<ScreenRow> {
    let across = tiles_across(layout, width);
    let mut lines = Vec::new();
    while !tiles.is_empty() {
        let rest = tiles.split_off(across.min(tiles.len()));
        let line = std::mem::replace(&mut tiles, rest);
        lines.push(ScreenRow { grid_line: true, ..ScreenRow::untitled(format!("grid-{}", lines.len()), layout, line) });
    }
    lines
}

#[cfg(test)]
mod tests {
    use max_api::cms::TileDetail;

    use super::*;

    fn tiles(count: usize) -> Vec<ScreenTile> {
        (0..count)
            .map(|number| ScreenTile {
                title: number.to_string(),
                route: None,
                artwork: None,
                logo: None,
                badge_icon: None,
                banner_icon: None,
                detail: TileDetail::default(),
            })
            .collect()
    }

    #[test]
    fn a_wider_window_fits_more_tiles_and_a_narrow_one_still_fits_one() {
        assert_eq!(tiles_across(Layout::Landscape, 1600.0), 4);
        assert_eq!(tiles_across(Layout::Landscape, 2000.0), 5);
        assert_eq!(tiles_across(Layout::Landscape, 200.0), 1);
    }

    #[test]
    fn cuts_a_run_of_tiles_into_full_lines_and_a_remainder() {
        let lines = grid_lines(tiles(10), Layout::Landscape, 1600.0);
        assert_eq!(lines.iter().map(|line| line.tiles.len()).collect::<Vec<_>>(), [4, 4, 2]);
        assert!(lines.iter().all(|line| line.grid_line && line.title.is_empty()));
        assert_eq!(lines[2].tiles[1].title, "9");
    }
}
