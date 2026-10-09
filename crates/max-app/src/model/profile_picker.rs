use max_api::client::{Account, Profile};

use super::account::ScreenAccount;

pub struct PinEntry {
    pub profile: usize,
    pub digits: String,
    pub focus_field: bool,
}

#[derive(Default)]
pub struct ProfilePicker {
    pub account: Option<ScreenAccount>,
    pub pin_entry: Option<PinEntry>,
    pub switching: bool,
}

#[derive(Debug, PartialEq)]
pub enum Chosen {
    AlreadyInUse,
    AsksForItsPin,
    Switch(ProfileSwitch),
}

/// What the service is asked for when the viewer changes profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ProfileSwitch {
    pub account: Account,
    pub profile_id: String,
    pub pin: Option<String>,
}

impl ProfilePicker {
    fn profile(&self, index: usize) -> Option<&Profile> {
        self.account.as_ref()?.account.profiles.get(index)
    }

    /// A profile guarded by a PIN is switched to only once the PIN is entered.
    pub fn choose(&mut self, index: usize) -> Option<Chosen> {
        let profile = self.profile(index)?;
        if profile.selected {
            Some(Chosen::AlreadyInUse)
        } else if profile.needs_pin {
            self.pin_entry = Some(PinEntry { profile: index, digits: String::new(), focus_field: true });
            Some(Chosen::AsksForItsPin)
        } else {
            self.switch_to(index, None).map(Chosen::Switch)
        }
    }

    pub fn submit_pin(&mut self) -> Option<ProfileSwitch> {
        let entry = self.pin_entry.take()?;
        self.switch_to(entry.profile, Some(entry.digits))
    }

    fn switch_to(&mut self, index: usize, pin: Option<String>) -> Option<ProfileSwitch> {
        let switch = ProfileSwitch { account: self.account.as_ref()?.account.clone(), profile_id: self.profile(index)?.id.clone(), pin };
        self.switching = true;
        Some(switch)
    }
}

#[cfg(test)]
mod tests;
