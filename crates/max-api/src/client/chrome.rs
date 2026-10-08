use super::{pages::COLLECTION_QUERY, Client};
use crate::{
    cms::{Document, NavigationItem},
    Error,
};

#[derive(Debug, Default)]
pub struct Chrome {
    pub navigation: Vec<NavigationItem>,
    pub logo: Option<String>,
    pub search_icon: Option<String>,
    pub my_stuff_icon: Option<String>,
    pub avatar: Option<String>,
}

impl Chrome {
    pub fn from_navigation_menu(document: &Document) -> Self {
        let icon = |name: &str| document.image_named(name).map(|image| image.source);
        Self {
            navigation: document.navigation_items(),
            logo: icon("ctv-branding-logo-max"),
            search_icon: icon("mobile-max-search-default"),
            my_stuff_icon: icon("mobile-my-stuff-icon-default"),
            avatar: None,
        }
    }
}

impl Client {
    pub fn chrome(&self) -> Result<Chrome, Error> {
        let url = format!("{}/cms/collections/navigation-menu-web?{COLLECTION_QUERY}", self.session.content_origin());
        let mut chrome = Chrome::from_navigation_menu(&Document::parse(&self.get(&url)?)?);
        match self.selected_avatar() {
            Ok(avatar) => chrome.avatar = avatar,
            Err(error) => eprintln!("profile picture failed: {error}"),
        }
        Ok(chrome)
    }

    fn selected_avatar(&self) -> Result<Option<String>, Error> {
        Ok(self.account()?.selected_profile().and_then(|profile| profile.avatar.clone()))
    }
}
