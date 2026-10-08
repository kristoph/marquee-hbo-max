use std::thread;

use eframe::egui;
use max_api::cms::MenuAction;

use super::App;
use crate::{
    intent::{AccountChange, AccountChangeKind},
    message::Message,
};

struct Request {
    method: &'static str,
    url: String,
    value: Option<String>,
    feedback: String,
}

impl App {
    pub(crate) fn change_account(&mut self, ctx: &egui::Context, change: AccountChange) {
        let AccountChange { tile: at, kind } = change;
        let Some(tile) = self.page.rows.get_mut(at.row).and_then(|row| row.tiles.get_mut(at.column)) else { return };
        let Some(request) = tile.detail.menu.iter_mut().find_map(|entry| request_for(&kind, entry)) else { return };
        if matches!(kind, AccountChangeKind::RemoveFromRow) {
            self.page.rows[at.row].tiles.remove(at.column);
            self.page.settle_selection();
            self.page.selected.column = self.page.selected.column.min(self.page.rows[self.page.selected.row].tiles.len().saturating_sub(1));
        }
        if !request.feedback.is_empty() {
            self.say(request.feedback.clone());
        }
        self.forget_visited_pages();
        let Some(client) = self.service.client() else { return };
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        thread::spawn(move || {
            if let Err(error) = client.change(request.method, &request.url, request.value.as_deref()) {
                let _ = sender.send(Message::AccountChangeFailed(error));
                ctx.request_repaint();
            }
        });
    }
}

fn request_for(kind: &AccountChangeKind, entry: &mut MenuAction) -> Option<Request> {
    match (kind, entry) {
        (AccountChangeKind::ToggleMyList, MenuAction::MyList { url, listed, added, removed, .. }) => {
            let (method, feedback) = if *listed { ("DELETE", removed.clone()) } else { ("POST", added.clone()) };
            *listed = !*listed;
            Some(Request { method, url: url.clone(), value: None, feedback })
        }
        (AccountChangeKind::Rate(value), MenuAction::Rate { url, chosen, .. }) => {
            let clearing = chosen.as_ref() == Some(value);
            *chosen = (!clearing).then(|| value.clone());
            Some(match clearing {
                true => Request { method: "DELETE", url: url.clone(), value: None, feedback: "Rating cleared".to_string() },
                false => Request { method: "PUT", url: url.clone(), value: Some(value.clone()), feedback: "Rating saved".to_string() },
            })
        }
        (AccountChangeKind::RemoveFromRow, MenuAction::Remove { url, done, .. }) => {
            Some(Request { method: "DELETE", url: url.clone(), value: None, feedback: done.clone() })
        }
        _ => None,
    }
}
