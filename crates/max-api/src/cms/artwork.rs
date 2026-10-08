use super::{
    image::{ICON_WIDTH, LOGO_WIDTH},
    layout::Layout,
    page::Row,
};

pub fn artwork_wanted(rows: &[Row]) -> Vec<(String, u32)> {
    let mut wanted = Vec::new();
    for row in rows {
        let layout = row.layout();
        wanted.extend(row.masthead.as_ref().map(|masthead| (masthead.source.clone(), LOGO_WIDTH)));
        wanted.extend(row.rank_images.iter().chain(&row.rank_images_selected).map(|image| (image.source.clone(), ICON_WIDTH)));
        for tile in &row.tiles {
            wanted.extend(tile.artwork(layout).map(|artwork| (artwork.source.clone(), layout.fetch_width())));
            if layout == Layout::Hero {
                wanted.extend(tile.logo().map(|logo| (logo.source.clone(), LOGO_WIDTH)));
            }
        }
    }
    wanted.sort();
    wanted.dedup();
    wanted
}
