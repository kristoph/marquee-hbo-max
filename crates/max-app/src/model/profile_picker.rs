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
mod tests {
    use super::*;

    fn picker() -> ProfilePicker {
        let profile =
            |id: &str, selected: bool, needs_pin: bool| Profile { id: id.to_string(), name: id.to_string(), avatar: None, needs_pin, selected };
        let profiles = vec![profile("mine", true, false), profile("kids", false, false), profile("guarded", false, true)];
        let account = Account { user_id: "user".to_string(), signed_in: true, profiles };
        ProfilePicker { account: Some(ScreenAccount { account, avatars: Vec::new() }), ..ProfilePicker::default() }
    }

    #[test]
    fn the_profile_in_use_is_not_switched_to() {
        let mut picker = picker();
        assert_eq!(picker.choose(0), Some(Chosen::AlreadyInUse));
        assert!(!picker.switching);
    }

    #[test]
    fn an_open_profile_is_switched_to_at_once() {
        let mut picker = picker();
        let Some(Chosen::Switch(switch)) = picker.choose(1) else { panic!("the profile should be switched to") };
        assert_eq!((switch.profile_id.as_str(), switch.pin), ("kids", None));
        assert!(picker.switching);
    }

    #[test]
    fn a_guarded_profile_waits_for_its_pin() {
        let mut picker = picker();
        assert_eq!(picker.choose(2), Some(Chosen::AsksForItsPin));
        assert!(!picker.switching);
        picker.pin_entry.as_mut().unwrap().digits.push_str("1234");
        let switch = picker.submit_pin().unwrap();
        assert_eq!((switch.profile_id.as_str(), switch.pin.as_deref()), ("guarded", Some("1234")));
        assert!(picker.pin_entry.is_none() && picker.switching);
    }

    #[test]
    fn nothing_is_chosen_before_the_profiles_have_loaded() {
        assert_eq!(ProfilePicker::default().choose(0), None);
        assert_eq!(ProfilePicker::default().submit_pin(), None);
    }
}
