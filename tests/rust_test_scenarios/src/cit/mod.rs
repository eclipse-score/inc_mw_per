use crate::cit::default_values::default_values_group;
use crate::cit::multiple_kvs::multiple_kvs_group;
use crate::cit::persistency::persistency_group;
use crate::cit::supported_datatypes::supported_datatypes_group;
use test_scenarios_rust::scenario::{ScenarioGroup, ScenarioGroupImpl};

mod default_values;
mod multiple_kvs;
mod persistency;
mod supported_datatypes;

pub fn cit_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "cit",
        vec![],
        vec![
            default_values_group(),
            multiple_kvs_group(),
            persistency_group(),
            supported_datatypes_group(),
        ],
    ))
}
