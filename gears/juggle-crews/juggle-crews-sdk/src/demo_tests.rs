use super::*;

#[test]
fn every_demo_user_belongs_to_a_demo_crew_with_one_owner() {
    for crew in CREWS {
        let owners = USERS
            .iter()
            .filter(|u| u.crew == crew.key && u.owner)
            .count();
        assert_eq!(owners, 1, "{} needs exactly one owner", crew.key);
    }
    assert!(USERS.iter().all(|u| CREWS.iter().any(|c| c.key == u.crew)));
    assert_ne!(crew_id("flat-4b"), crew_id("robotics-club"));
}
