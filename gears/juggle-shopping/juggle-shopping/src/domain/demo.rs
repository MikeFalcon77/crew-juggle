//! The demo crews' shopping lists, rebuilt on every start in demo mode.
use super::error::Result;
use super::model::{ExtractedProduct, State};
use uuid::Uuid;

/// One demo crew's list.
#[toolkit::domain_model]
pub struct DemoList {
    /// Demo crew key (see `juggle_crews_sdk::demo`).
    pub crew: &'static str,
    /// Who added the items.
    pub actor: &'static str,
    /// (name, category, subcategory, quantity)
    pub items: &'static [(&'static str, &'static str, &'static str, &'static str)],
}

/// Both demo crews' lists.
pub const LISTS: [DemoList; 2] = [
    DemoList {
        crew: "flat-4b",
        actor: "Pip",
        items: &[
            ("Cat food", "pets", "Pet food", "6 pouches"),
            ("Cat litter", "pets", "Pet supplies", "1 bag"),
            ("Dish soap", "home", "Cleaning", "1"),
            ("Oat milk", "food", "Drinks", "2"),
            ("Instant noodles", "food", "Snacks", "5"),
        ],
    },
    DemoList {
        crew: "robotics-club",
        actor: "Ike",
        items: &[
            ("Pretzels", "food", "Snacks", "3 bags"),
            ("Energy drinks", "food", "Drinks", "6"),
            ("Resistor kit", "hobbies", "Electronics & parts", "1"),
            ("Zip ties", "hobbies", "Crafts", "1 pack"),
        ],
    },
];

/// Adds a demo list to a fresh state. Item ids are stable across restarts.
pub fn fill(state: &mut State, list: &DemoList) -> Result<()> {
    for (i, (name, category, subcategory, quantity)) in list.items.iter().enumerate() {
        let id = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("crew-juggle/demo/{}/item/{i}", list.crew).as_bytes(),
        );
        state.add_classified(
            id.to_string(),
            &ExtractedProduct {
                name: (*name).into(),
                product_id: None,
                category: (*category).into(),
                subcategory: (*subcategory).into(),
                quantity: (*quantity).into(),
                variant: String::new(),
                original_name: (*name).into(),
            },
            list.actor,
        )?;
    }
    Ok(())
}
