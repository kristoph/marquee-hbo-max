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
mod tests {
    use super::*;

    fn responses() -> (Value, Value) {
        let user = json!({"data": {"id": "u1", "attributes": {"selectedProfileId": "p2", "anonymous": false}}});
        let profiles = json!({
            "data": [
                {"id": "p1", "attributes": {"profileName": "Ada", "pinRestricted": true},
                 "relationships": {"avatar": {"data": {"id": "a1", "type": "avatar"}}}},
                {"id": "p2", "attributes": {"profileName": "Kit"},
                 "relationships": {"avatar": {"data": {"id": "a2", "type": "avatar"}}}}
            ],
            "included": [
                {"type": "avatar", "id": "a1", "relationships": {"avatarImage": {"data": {"id": "i1", "type": "image"}}}},
                {"type": "avatar", "id": "a2", "relationships": {"avatarImage": {"data": {"id": "i2", "type": "image"}}}},
                {"type": "image", "id": "i1", "attributes": {"src": "https://img.example/one.png"}},
                {"type": "image", "id": "i2", "attributes": {"src": "https://img.example/two.png"}}
            ]
        });
        (user, profiles)
    }

    #[test]
    fn reads_profiles_with_their_avatars_and_which_is_selected() {
        let (user, profiles) = responses();
        let account = Account::from_responses(&user, &profiles).unwrap();
        assert_eq!((account.user_id.as_str(), account.signed_in), ("u1", true));
        assert_eq!(
            account.profiles.iter().map(|profile| (profile.name.as_str(), profile.needs_pin, profile.selected)).collect::<Vec<_>>(),
            [("Ada", true, false), ("Kit", false, true)]
        );
        assert_eq!(account.selected_profile().unwrap().avatar.as_deref(), Some("https://img.example/two.png"));
    }

    #[test]
    fn an_anonymous_visitor_is_not_signed_in() {
        let (_, profiles) = responses();
        let visitor = json!({"data": {"id": "u0", "attributes": {"anonymous": true}}});
        assert!(!Account::from_responses(&visitor, &profiles).unwrap().signed_in);
        assert_eq!(Account::from_responses(&json!({}), &profiles), None);
    }
}
