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
