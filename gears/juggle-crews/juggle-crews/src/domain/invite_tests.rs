use super::*;

#[test]
fn invite_codes_ignore_case_spaces_and_dashes() {
    assert_eq!(normalize_code(" k7qx-m2pa "), "K7QXM2PA");
    let code = new_code().unwrap();
    assert_eq!(code.len(), 8);
    assert!(code.bytes().all(|b| CODE_ALPHABET.contains(&b)));
}

#[test]
fn codes_are_shown_with_a_dash_in_the_middle() {
    assert_eq!(display_code("K7QXM2PA"), "K7QX-M2PA");
}
