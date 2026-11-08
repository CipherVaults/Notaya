//! Attribution supplied by the trusted caller, never by an action's parameters.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Actor {
    Person,
    Automation,
    Ai { pairing_id: String, label: String },
}
