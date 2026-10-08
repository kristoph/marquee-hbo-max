use max_api::cms::{Page, TileKind};

pub fn print(page: &Page, fetched_live: bool) {
    println!("{} — {} rows", page.title, page.rows.len());
    for row in &page.rows {
        let layout = match &row.template {
            Some(template) => format!("{}/{template}", row.component),
            None => row.component.clone(),
        };
        println!("\n{}  [{layout}]", row.title);
        if row.tiles.is_empty() {
            println!("    ({})", if row.deferred && !fetched_live { "loaded separately" } else { "empty" });
        }
        for tile in &row.tiles {
            println!("    {:<10} {:<44} {}", kind_name(tile.kind), tile.title, tile.route.as_deref().unwrap_or("-"));
        }
    }
}

fn kind_name(kind: TileKind) -> &'static str {
    match kind {
        TileKind::Show => "show",
        TileKind::Video => "video",
        TileKind::Link => "link",
        TileKind::Category => "category",
        TileKind::Collection => "collection",
        TileKind::Channel => "channel",
        TileKind::View => "view",
    }
}
