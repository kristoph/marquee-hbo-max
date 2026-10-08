use eframe::egui::Vec2;
use max_api::{
    client::{Account, Client},
    Error,
};

use super::chrome::prepare_image;
use crate::metrics::profiles::AVATAR;

const AVATAR_FETCH_WIDTH: u32 = 400;

pub struct ScreenAccount {
    pub account: Account,
    pub avatars: Vec<Option<String>>,
}

pub fn load_account(client: &Client) -> Result<ScreenAccount, Error> {
    let account = client.account()?;
    let avatars = account.profiles.iter().map(|profile| prepare_image(profile.avatar.as_ref()?, AVATAR_FETCH_WIDTH, Vec2::splat(AVATAR))).collect();
    Ok(ScreenAccount { account, avatars })
}
