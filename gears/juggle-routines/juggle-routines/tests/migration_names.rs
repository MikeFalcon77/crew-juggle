//! Applied migrations are recorded by name, so a rename would run them again
//! on an existing database. This pins the names the first release shipped.
use toolkit::DatabaseCapability;

#[test]
fn migration_names_never_change() {
    let names: Vec<String> = juggle_routines::RoutinesGear::default()
        .migrations()
        .iter()
        .map(|migration| migration.name().to_owned())
        .collect();
    assert_eq!(names, ["storage"]);
}
