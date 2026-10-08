use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Sender,
    },
    thread,
};

use eframe::egui;
use max_api::{
    client::{captured_page, Client, HOME_ROUTE},
    cms::Layout,
    Error,
};

use crate::{
    artwork,
    message::Message,
    model::{build_page, Download, Outstanding, PendingRow, ScreenBuilder, ScreenPage, ScreenRow},
    paths,
    service::Service,
};

const ROW_FETCHERS: usize = 4;

pub fn load_page(service: &Service, route: &str) -> Result<(ScreenPage, Outstanding), Error> {
    let page = match service {
        Service::Live(client) => client.page_outline(route)?,
        Service::Captured if route == HOME_ROUTE => captured_page(paths::captured_home())?,
        Service::Captured => return Err(Error::NotFound),
        Service::SignedOut => return Err(Error::SignedOut),
    };
    Ok(build_page(page, service.is_live()))
}

pub fn fetch_outstanding(outstanding: Outstanding, client: Option<&Client>, sender: &Sender<Message>, ctx: &egui::Context) {
    let Outstanding { downloads, rows } = outstanding;
    let next_row = AtomicUsize::new(0);
    thread::scope(|scope| {
        scope.spawn(|| download(&downloads, ctx));
        for _ in 0..ROW_FETCHERS.min(rows.len()) {
            let sender = sender.clone();
            let (rows, next_row) = (&rows, &next_row);
            scope.spawn(move || {
                while let Some(pending) = rows.get(next_row.fetch_add(1, Ordering::Relaxed)) {
                    let mut builder = ScreenBuilder::new(true);
                    let _ = sender.send(fetch_row(client, pending, &mut builder));
                    ctx.request_repaint();
                    download(&builder.outstanding.downloads, ctx);
                }
            });
        }
    });
}

/// A row that could not be fetched still answers for its place on the page, as an empty one.
fn fetch_row(client: Option<&Client>, pending: &PendingRow, builder: &mut ScreenBuilder) -> Message {
    let fetched = client.ok_or(Error::SignedOut).and_then(|client| client.row(&pending.id, pending.mandatory_parameters.as_deref()));
    let settled = |row: ScreenRow| ScreenRow { id: pending.id.clone(), pending: false, ..row };
    match fetched {
        Ok(row) => Message::Row(settled(builder.row(&row))),
        Err(error) => Message::RowFailed { row: settled(ScreenRow::untitled(String::new(), Layout::Other, Vec::new())), error },
    }
}

fn download(downloads: &[Download], ctx: &egui::Context) {
    let wanted: Vec<(String, u32)> = downloads.iter().map(|download| (download.source.clone(), download.width)).collect();
    artwork::cache().fetch_each(&wanted, |index, result| match result {
        Ok(_) => {
            downloads[index].mark_ready();
            ctx.request_repaint();
        }
        Err(error) => log::warn!("image failed: {error}"),
    });
}
