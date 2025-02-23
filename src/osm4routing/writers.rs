use super::models::*;

pub fn csv(nodes: Vec<Node>, edges: Vec<Edge>) {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let edges_filename = format!("edges_{}.csv", today);
    let edges_path = std::path::Path::new(&edges_filename);
    // add today's ate to the filename
    let mut edges_csv = csv::Writer::from_path(edges_path).unwrap();
    edges_csv
        .serialize(vec![
            "id",
            "osm_id",
            "source",
            "target",
            "length",
            "foot",
            "car_forward",
            "car_backward",
            "bike_forward",
            "bike_backward",
            "train",
            "wkt",
            "tags"
        ])
        .expect("CSV: unable to write edge header");
    for edge in edges {
        edges_csv
            .serialize((
                &edge.id,
                edge.osm_id.0,
                edge.source.0,
                edge.target.0,
                edge.length(),
                edge.properties.foot,
                edge.properties.car_forward,
                edge.properties.car_backward,
                edge.properties.bike_forward,
                edge.properties.bike_backward,
                edge.properties.train,
                edge.as_wkt(),
                // write tags as json
                serde_json::to_string(&edge.tags).unwrap_or("".to_string()),
            ))
            .expect("CSV: unable to write edge");
    }
    let nodes_filename = format!("nodes_{}.csv", today);
    let nodes_path = std::path::Path::new(&nodes_filename);
    let mut nodes_csv = csv::Writer::from_path(nodes_path).unwrap();
    nodes_csv
        .serialize(vec!["id", "lon", "lat"])
        .expect("CSV: unable to write node header");
    for node in nodes {
        nodes_csv
            .serialize((node.id.0, node.coord.lon, node.coord.lat))
            .expect("CSV: unable to write node");
    }
}

// pub fn pg(nodes: Vec<Node>, edges: Vec<Edge>) {}
