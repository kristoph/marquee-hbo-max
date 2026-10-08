use max_api::cms::{Image, Layout, Row, Tile, ICON_WIDTH, LOGO_WIDTH};

use super::{
    lazy_image::{Download, LazyImage, SizedImage},
    rail::{RankNumeral, ScreenRow, ScreenTile},
};
use crate::artwork;

pub struct PendingRow {
    pub id: String,
    pub mandatory_parameters: Option<String>,
}

#[derive(Default)]
pub struct Outstanding {
    pub downloads: Vec<Download>,
    pub rows: Vec<PendingRow>,
}

pub struct ScreenBuilder {
    fetch_deferred_rows: bool,
    pub outstanding: Outstanding,
}

impl ScreenBuilder {
    pub fn new(fetch_deferred_rows: bool) -> Self {
        Self { fetch_deferred_rows, outstanding: Outstanding::default() }
    }

    pub fn rows(&mut self, rows: &[Row]) -> Vec<ScreenRow> {
        rows.iter().map(|row| self.row(row)).collect()
    }

    pub fn row(&mut self, row: &Row) -> ScreenRow {
        let layout = row.layout();
        let tiles: Vec<ScreenTile> = row.tiles.iter().map(|tile| self.tile(tile, row, layout)).collect();
        let pending = self.fetch_deferred_rows && row.deferred && tiles.is_empty();
        if pending {
            self.outstanding.rows.push(PendingRow { id: row.id.clone(), mandatory_parameters: row.mandatory_parameters.clone() });
        }
        let rank_numerals = (0..row.rank_images.len())
            .map(|place| RankNumeral {
                plain: row.rank_images.get(place).and_then(|image| self.image(&image.source, ICON_WIDTH)),
                selected: row.rank_images_selected.get(place).and_then(|image| self.image(&image.source, ICON_WIDTH)),
            })
            .collect();
        ScreenRow {
            id: row.id.clone(),
            title: row.title.clone(),
            layout,
            numbered: row.is_numbered(),
            masthead: row.masthead.as_ref().and_then(|masthead| self.sized(masthead, LOGO_WIDTH)),
            rank_numerals,
            tiles,
            pending,
            grid_line: false,
            resumes_watching: row.resumes_watching(),
        }
    }

    fn tile(&mut self, tile: &Tile, row: &Row, layout: Layout) -> ScreenTile {
        let show_artwork = if row.use_show_artwork { tile.show_artwork(layout) } else { None };
        let artwork = show_artwork.or_else(|| tile.artwork(layout));
        let logo = tile.logo().filter(|_| layout == Layout::Hero);
        let icon_of = |badge: &Option<max_api::cms::Badge>| badge.as_ref().and_then(|badge| badge.icon.clone());
        ScreenTile {
            title: tile.title.clone(),
            route: tile.route.clone(),
            artwork: artwork.and_then(|artwork| self.image(&artwork.source, layout.fetch_width())),
            logo: logo.and_then(|logo| self.sized(logo, LOGO_WIDTH)),
            badge_icon: icon_of(&tile.detail.badge).and_then(|icon| self.sized(&icon, ICON_WIDTH)),
            banner_icon: icon_of(&tile.detail.banner).and_then(|icon| self.sized(&icon, ICON_WIDTH)),
            detail: tile.detail.clone(),
        }
    }

    fn image(&mut self, source: &str, width: u32) -> Option<LazyImage> {
        let path = std::path::absolute(artwork::cache().path_for(source, width)).ok()?;
        let image = LazyImage::cached_at(&path);
        if !image.is_ready() {
            self.outstanding.downloads.push(image.download(source, width));
        }
        Some(image)
    }

    fn sized(&mut self, image: &Image, width: u32) -> Option<SizedImage> {
        Some(SizedImage { image: self.image(&image.source, width)?, aspect: image.aspect() })
    }
}
