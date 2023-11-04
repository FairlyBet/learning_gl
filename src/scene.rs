use std::fs::File;
use serde::Deserialize;
// use serde_json:: 

const GRAPH_FILE: &str = "scene-graph.json";

pub fn read_graph() {
    let mut file = File::open(GRAPH_FILE).unwrap();

}

struct Object {

}
