use super::*;

fn picker() -> ProfilePicker {
    let profile = |id: &str, selected: bool, needs_pin: bool| Profile { id: id.to_string(), name: id.to_string(), avatar: None, needs_pin, selected };
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
