use grph::prelude::*;
use std::{fs, str::FromStr, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    nearest_neighbor("data/K_10.txt");
    // nearest_neighbor("data/K_10e.txt");
    // nearest_neighbor("data/K_12.txt");
    // nearest_neighbor("data/K_12e.txt");
    // nearest_neighbor("data/K_15.txt");
    // nearest_neighbor("data/K_15e.txt");
    // nearest_neighbor("data/K_20.txt");
    // nearest_neighbor("data/K_30.txt");
    // nearest_neighbor("data/K_50.txt");
    // nearest_neighbor("data/K_70.txt");
    // nearest_neighbor("data/K_100.txt");

    double_tree("data/K_10.txt");
    // double_tree("data/K_10e.txt");
    // double_tree("data/K_12.txt");
    // double_tree("data/K_12e.txt");
    // double_tree("data/K_15.txt");
    // double_tree("data/K_15e.txt");
    // double_tree("data/K_20.txt");
    // double_tree("data/K_30.txt");
    // double_tree("data/K_50.txt");
    // double_tree("data/K_70.txt");
    // double_tree("data/K_100.txt");

    branch_bound("data/K_10.txt");

    brute_force("data/K_10.txt");

    // db();

    // depth_search("data/K_10.txt");

    Ok(())
}

pub fn brute_force(path: &str) {
    let edge_list = fs::read_to_string(path).unwrap();
    let edge_list = EdgeList::from_str(&edge_list).unwrap();
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();

    let now = Instant::now();

    let total = graph.brute_force().unwrap().weight as f32;

    println!("bf: {path}: {total} in {:?}", now.elapsed());
}

pub fn branch_bound(path: &str) {
    let edge_list = fs::read_to_string(path).unwrap();
    let edge_list = EdgeList::from_str(&edge_list).unwrap();
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();

    let now = Instant::now();

    let total = graph.branch_bound().unwrap().weight as f32;

    println!("bb: {path}: {total} in {:?}", now.elapsed());
}

pub fn nearest_neighbor(path: &str) {
    let edge_list = fs::read_to_string(path).unwrap();
    let edge_list = EdgeList::from_str(&edge_list).unwrap();
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();
    let total = graph.nearest_neighbor_from_first().unwrap().weight as f32;

    let now = Instant::now();

    println!("nn: {path}: {total} in {:?}", now.elapsed());
}

pub fn double_tree(path: &str) {
    let edge_list = fs::read_to_string(path).unwrap();
    let edge_list = EdgeList::from_str(&edge_list).unwrap();
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();
    let total = graph.double_tree().unwrap().weight as f32;

    let now = Instant::now();

    println!("dt: {path}: {total} in {:?}", now.elapsed());
}

pub fn db() {
    let edge_list = EdgeList::with(
        [
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 3, 1.0),
            (3, 0, 10.0),
            (0, 2, 2.0),
            (1, 3, 2.0),
        ]
        .into_iter(),
        4,
    );
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();
    let total = graph.double_tree().unwrap().weight;

    println!("{total}")
}

pub fn nn() {
    let edge_list = EdgeList::with(
        [
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 3, 1.0),
            (3, 0, 10.0),
            (0, 2, 2.0),
            (1, 3, 2.0),
        ]
        .into_iter(),
        4,
    );
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();
    let nn = graph.nearest_neighbor_from_first().unwrap().weight;

    println!("{nn}")
}

pub fn prim() {
    let edge_list = EdgeList::with(
        [
            (1, 2, 0.2),
            (1, 3, 0.3),
            (1, 4, 0.3),
            (2, 3, 0.4),
            (2, 5, 0.3),
            (3, 4, 0.5),
            (3, 5, 0.1),
            (4, 6, 0.7),
            (5, 6, 0.8),
            (6, 0, 0.9),
        ]
        .into_iter(),
        7,
    );
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();
    let total = graph.prim();

    assert_eq!(total, 2.5);
}

pub fn graph_gross() -> Result<(), Box<dyn std::error::Error>> {
    let edge_list = fs::read_to_string("data/Graph_ganzgross.txt")?;
    let edge_list = EdgeList::from_str(&edge_list).unwrap();
    let graph = AdjacencyList::<usize, ()>::try_from(edge_list).unwrap();

    let now = Instant::now();
    let counter = graph.bfs_connected_components().count();
    let elapsed = now.elapsed();

    println!("Counter: {counter} in {:?}", elapsed);

    Ok(())
}

pub fn depth_search(path: &str) {
    let edge_list = fs::read_to_string(path).unwrap();
    let edge_list = EdgeList::from_str(&edge_list).unwrap();
    let graph = AdjacencyList::<usize, f64>::try_from(edge_list).unwrap();

    let connected_components = graph.dfs_connected_components().count();

    println!("Connected Components: {connected_components}");
}
