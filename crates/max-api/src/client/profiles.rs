use serde_json::{json, Value};

use super::{Body, Client};
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub needs_pin: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub user_id: String,
    pub signed_in: bool,
    pub profiles: Vec<Profile>,
}

impl Account {
    pub fn selected_profile(&self) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.selected)
    }

    fn from_responses(user: &Value, profiles: &Value) -> Option<Self> {
        let attributes = &user["data"]["attributes"];
        let selected_id = attributes["selectedProfileId"].as_str();
        let included =
            |kind: &str, id: &Value| profiles["included"].as_array()?.iter().find(|resource| resource["type"] == kind && &resource["id"] == id);
        let avatar_of = |profile: &Value| {
            let avatar = included("avatar", &profile["relationships"]["avatar"]["data"]["id"])?;
            let image = included("image", &avatar["relationships"]["avatarImage"]["data"]["id"])?;
            image["attributes"]["src"].as_str().map(str::to_string)
        };
        let profiles = profiles["data"]
            .as_array()?
            .iter()
            .filter_map(|profile| {
                let id = profile["id"].as_str()?;
                Some(Profile {
                    id: id.to_string(),
                    name: profile["attributes"]["profileName"].as_str().unwrap_or_default().to_string(),
                    avatar: avatar_of(profile),
                    needs_pin: profile["attributes"]["pinRestricted"].as_bool().unwrap_or(false),
                    selected: selected_id == Some(id),
                })
            })
            .collect();
        Some(Self { user_id: user["data"]["id"].as_str()?.to_string(), signed_in: attributes["anonymous"].as_bool() == Some(false), profiles })
    }
}

impl Client {
    pub fn account(&self) -> Result<Account, Error> {
        let origin = self.session.account_origin();
        let user: Value = serde_json::from_str(&self.get(&format!("{origin}/users/me"))?)?;
        let profiles: Value = serde_json::from_str(&self.get(&format!("{origin}/users/me/profiles"))?)?;
        Account::from_responses(&user, &profiles).ok_or(Error::Lacks("an account"))
    }

    /// The service keeps the selected profile with the session, so the same session goes on
    /// working, for the embedded player too.
    pub fn switch_profile(&self, account: &Account, profile_id: &str, pin: Option<&str>) -> Result<(), Error> {
        let request = json!({
            "data": {"attributes": {"selectedProfileId": profile_id, "profilePin": pin}, "id": account.user_id, "type": "user"}
        });
        let url = format!("{}/users/me/profiles/switchProfile", self.session.account_origin());
        self.send("POST", &url, Body::Json(&request.to_string()))?;
        Ok(())
    }

    pub fn sign_out(&self) -> Result<(), Error> {
        self.send("POST", &format!("{}/logout", self.session.account_origin()), Body::Empty)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
