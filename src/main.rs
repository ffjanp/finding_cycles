extern crate rand;
extern crate rand_xoshiro;
use std::fs::File;
use std::io::{ BufReader,Error, BufWriter,stdout,stdin};
use std::collections::HashMap;
use std::iter::Sum;
use std::env;
use std::thread;
use std::cmp;
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;
use rand::distributions::{Distribution, Standard};

fn read_number<R: std::io::BufRead>(io:&mut R,m:usize) -> Result<Vec<usize>,Error> {
    let mut v = vec![]; for _ in 0..m {
        let mut line = String::new();
        io.read_line(&mut line)?; 
        let n: usize = line.trim().parse::<usize>().unwrap(); 
        v.push(n);
        v.sort_unstable();
    }
    Ok(v)
}

fn read_edges<R: std::io::BufRead>(io:&mut R,m:usize) -> Result<Vec<(Vec<usize>,Vec<f32>)>,Error> {
    let mut v = vec![];
    for _ in 0..m {
        let mut line = String::new();
        io.read_line(&mut line)?;
        let splitted: Vec<&str> = line.split(';').collect();
        let edge: Vec<usize> = splitted[0].split(' ').map(|x| x.trim().parse::<usize>().unwrap()).collect();
        let mut weights: Vec<f32> = vec![];
        if splitted.len() > 1 {
            weights = splitted[1].split(' ').map(|x| x.trim().parse::<f32>().unwrap()).collect();
        }
        v.push((edge,weights));
    }
    Ok(v)
}

fn remove_edges(graph:&mut HashMap<usize,Vec<usize>>,node : usize) {
    for (_,likes) in &mut graph.iter_mut() {
        if likes.len() > 0 && likes[likes.len()-1] == node {
            likes.pop();
        }
    }
    graph.remove(&node);    
} 
fn find_cycles (graph:&HashMap<usize,Vec<usize>>,startnode : usize,n:usize) -> Vec<Vec<usize>>  {
    let mut path = vec![startnode];
    let mut stack = vec![(startnode,graph[&startnode].to_vec())];
    let mut cycles = vec![];
    while stack.len() > 0 {
        let l = stack.len();
        if stack[l - 1].1.len() > 0 {
            let nextnode = stack[l - 1].1.pop().unwrap();
            if nextnode == startnode {
//                out.write("{:?}",path);
//                println!("{:?}",path);
                cycles.push(path.to_vec());
            }
            else if !(path.contains(&nextnode)) && path.len() < n {
                path.push(nextnode);
                stack.push((nextnode,graph[&nextnode].to_vec()));
            }
            else if l == n -1 {
                stack.pop();
                path.pop();
            }
        }
        else if stack[l - 1].1.len() == 0 {
            stack.pop();
            path.pop();
        }
    } 
    cycles
}

fn make_subgraph(graph: &HashMap<usize,Vec<usize>>, cycle: &Vec<usize>) -> HashMap<usize,Vec<usize>> {
    let mut subgraph: HashMap<usize,Vec<usize>> = HashMap::new();
    for &node in cycle {
        subgraph.insert(node,graph[&node].iter().filter(|x| cycle.contains(x)).map(|&x| x).collect());
    }
    subgraph
}
fn random_remove_edges<T: Rng>(graph :&HashMap<usize,Vec<usize>>, weights: &HashMap<(usize,usize),f32>, random:&mut T) -> HashMap<usize,Vec<usize>> {
    let mut new_graph: HashMap<usize,Vec<usize>> = HashMap::new();
    for (&node,edges) in graph.iter() {
        let mut new_edges :Vec<usize>= vec![];
        for &edge in edges.iter() {
            let value: f32 = random.sample(Standard);
            if value < weights[&(node,edge)] {
                new_edges.push(edge);
            }
        new_graph.insert(node,new_edges.to_vec());
        }
    }
    new_graph
}
fn monte_carlo<T: Rng>(graph :&HashMap<usize,Vec<usize>>,weights: &HashMap<(usize,usize),f32>,mut random:&mut T,iterations: usize) -> f32 {
    let mut results : Vec<usize> = vec![];
    for i in 0..iterations {
        let new_graph = random_remove_edges(&graph , &weights, &mut random);
//        println!("----------------- here is removed graph");
//        println!("{:?}",new_graph);
        let mut length: usize = new_graph.len();
        let mut max:usize= 0;
        for &node in new_graph.keys() {
            let new_max:usize = find_cycles(&new_graph,node,length).iter().map(|x| x.len()).max().unwrap_or(0);
            max = cmp::max(max,new_max);
        }
        results.push(max); 
    }
    let result:usize = results.iter().sum();
    (result as f32) / ((graph.len() * iterations) as f32)
}



fn cycle_worker(this_worker: usize,total_workers:usize,nodes:Vec<usize>,edges: Vec<(Vec<usize>,Vec<f32>)>) {
     // READING THE COMMAND LINE ARGUMENTS 
    let args: Vec<String> = env::args().collect();
    let length :usize = args[1].parse().unwrap();

   
    //MAKING THE GRAPHS, PER NODE THE EDGES OUT ARE STORED in  a sorted list
    let mut graph: HashMap<usize,Vec<usize>> = HashMap::new();
    let mut weights: HashMap<(usize,usize),f32> = HashMap::new();
    for &node in &nodes {
        graph.insert(node,vec![]);
    }
    for edge in edges {
        graph.entry(edge.0[0]).or_insert(vec![]).push(edge.0[1]);
        weights.insert((edge.0[0],edge.0[1]),edge.1[0]);
    }
    for (_,likes) in &mut graph {
        likes.sort_unstable();
    }
    // Random Number generation
    let mut rng = Xoroshiro128StarStar::seed_from_u64(0); 
 
    // HERE THE NODES AND EDGES ARE REMOVED WHICH ARE SKIPPED
    //for &n in &nodes[0..begin] {
    //    remove_edges(&mut graph,n);
    //}
    // HERE ALL THE CYCLES OF LENGTH "length" STARTING AT THE NODES begin..end ARE FOUND AND 
    // PRINTED TO STANDARD OUT.
    let mut file = BufWriter::new(stdout()); 
    let mut i = 1;
    for &n in &nodes {
        if i % total_workers == this_worker { 
            let subgraphs:Vec<HashMap<usize,Vec<usize>>> = find_cycles(&graph,n,length).iter().map(|cycle| make_subgraph(&graph,cycle)).collect();
            //println!("{:?}",subgraphs);
            for subgraph in subgraphs.iter() {
                let nodes : Vec<usize> = subgraph.keys().map(|&x| x).collect();
                println!("{:?} ; {}",nodes,monte_carlo(&subgraph,&weights,&mut rng,1000));
            }
        }
        i = i + 1;
        remove_edges(&mut graph, n);
    } 

}

fn main() {
     // READING THE COMMAND LINE ARGUMENTS 
    let args: Vec<String> = env::args().collect();
    let n_workers :usize = args[2].parse().unwrap();
    //let mut f = BufReader::new(File::open(filename).expect("file not found"));
    let mut f = BufReader::new(stdin());

    // READING THE GRAPH FILE
    let graph_properties = read_number(&mut f,2).unwrap();
    let mut nodes = read_number(&mut f,graph_properties[0]).unwrap();
    let edges = read_edges(&mut f,graph_properties[1]).unwrap();
    nodes.reverse();
 
    // HERE WORK IS DELEGATED TO THE WORKERS
    let mut handles = vec![];
    for i in 0..n_workers {
        let nodes_temp = nodes.clone();
        let edges_temp = edges.clone();
        let handle = thread::spawn(move|| {
            cycle_worker(i,n_workers,nodes_temp ,edges_temp);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
} 
