extern crate rand;
extern crate rand_xoshiro;
extern crate clap;
use std::io::{ BufReader,Error, stdin};
use std::collections::HashMap;
use std::thread;
use std::cmp;
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;
use rand::distributions::Standard;
use clap::{Arg, App, SubCommand};

#[derive(Debug,Clone)]
struct DiGraph {
    nodes : Vec<usize>,
    edges : HashMap<usize,Vec<usize>>
}

impl DiGraph {
    fn new(mut nodes:Vec<usize>, edges:&Vec<(Vec<usize>,Vec<f32>)>) -> DiGraph {    
        let mut graph: HashMap<usize,Vec<usize>> = HashMap::new();
        for &node in &nodes { 
            graph.insert(node,vec![]);
        }
        for edge in edges {
            graph.entry(edge.0[0]).or_insert(vec![]).push(edge.0[1]);
        }
        for (_,likes) in &mut graph {
        likes.sort_unstable();
        }
        nodes.sort_unstable();
        DiGraph {
            nodes : nodes,
            edges : graph
        }
    }
    fn remove_node(&mut self) {
        let node = self.nodes.pop().unwrap();
        for (_,likes) in self.edges.iter_mut() {
            if likes.len() > 0 && likes[likes.len()-1] == node {
                likes.pop();
            }
        }
        self.edges.remove(&node);    
    } 


    fn make_subgraph(&mut self ,mut cycle: Vec<usize>) -> DiGraph {
        let mut subedges: HashMap<usize,Vec<usize>> = HashMap::new();
        for &node in &cycle {
            subedges.insert(node,self.edges[&node].iter().filter(|x| cycle.contains(x)).map(|&x| x).collect());
        }
        cycle.sort_unstable();
        DiGraph {
            nodes : cycle,
            edges : subedges
        }
    }
    fn find_cycles(&self,n:usize) -> Vec<Vec<usize>>  {
        let startnode = self.nodes.last().unwrap().clone();
        let mut path = vec![startnode];
        let mut stack = vec![(startnode,self.edges[&startnode].to_vec())];
        let mut cycles = vec![];
        while stack.len() > 0 {
            let l = stack.len();
            if stack[l - 1].1.len() > 0 {
                let nextnode = stack[l - 1].1.pop().unwrap();
                if nextnode == startnode {
                    cycles.push(path.to_vec());
                }
                else if !(path.contains(&nextnode)) && path.len() < n {
                    path.push(nextnode);
                    stack.push((nextnode,self.edges[&nextnode].to_vec()));
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


}    


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



fn store_weights(edges :&Vec<(Vec<usize>,Vec<f32>)>) -> HashMap<(usize,usize),f32> {
    let mut weights: HashMap<(usize,usize),f32> = HashMap::new();
    for edge in edges {
        weights.insert((edge.0[0],edge.0[1]),edge.1[0]);
    }
    weights
}

fn random_remove_edges<T: Rng>(graph :&DiGraph, weights: &HashMap<(usize,usize),f32>, random:&mut T) -> DiGraph {
    let mut new_edges: HashMap<usize,Vec<usize>> = HashMap::new();
    for (&node,edges) in graph.edges.iter() {
        let mut new_edge :Vec<usize>= vec![];
        for &edge in edges.iter() {
            let value: f32 = random.sample(Standard);
            if value < weights[&(node,edge)] {
                new_edge.push(edge);
            }
        new_edges.insert(node,new_edge.to_vec());
        }
    }
    DiGraph {
    nodes : graph.nodes.clone(),
    edges : new_edges
    }
}
fn monte_carlo<T: Rng>(graph :&DiGraph,weights: &HashMap<(usize,usize),f32>,mut random:&mut T,iterations: usize) -> f32 {
    let mut results : Vec<usize> = vec![];
    for _ in 0..iterations {
        let new_graph = random_remove_edges(&graph , &weights, &mut random);
//        println!("----------------- here is removed graph");
//        println!("{:?}",new_graph);
        let mut length: usize = new_graph.nodes.len();
        let mut max:usize= 0;
        for _ in &new_graph.nodes {
            let new_max:usize = new_graph.find_cycles(length).iter().map(|x| x.len()).max().unwrap_or(0);
            max = cmp::max(max,new_max);
        }
        results.push(max); 
    }
    let result:usize = results.iter().sum();
    (result as f32) / ((graph.nodes.len() * iterations) as f32)
}

fn calculate_chance(cycle :&Vec<usize>, weights : &HashMap<(usize,usize),f32>) -> f32 {
    let length = cycle.len();
    let mut product = 1.0;
    for i in 0..(length) {
        product *= weights[&(cycle[i],cycle[(i+1)%length])];
    }
    product 
}



fn cycle_worker(this_worker: usize,total_workers:usize,nodes:Vec<usize>,edges: Vec<(Vec<usize>,Vec<f32>)>,length:usize,mc_tests:usize) {
    //MAKING THE GRAPHS, PER NODE THE EDGES OUT ARE STORED in  a sorted list
    let mut graph = DiGraph::new(nodes.clone(),&edges);
    let weights = store_weights(&edges);

    // Random Number generation
    let mut rng = Xoroshiro128StarStar::seed_from_u64(0); 
    // Below we iterate over all the nodes, for every node we find the cycles starting at that
    // node, after which we delete the node from the graph. 
    let mut i = 1;
    for _ in &nodes {
        if i % total_workers == this_worker { 
            let subgraphs: Vec<(Vec<usize>,DiGraph)> = graph.find_cycles(length).iter().map(|cycle| (cycle.to_vec(),graph.make_subgraph(cycle.to_vec()))).collect();
            //println!("{:?}",subgraphs);
            let mut to_print = String::new();
            for (cycle,subgraph) in subgraphs.into_iter() {
                //let nodes : Vec<usize> = subgraph.nodes;
                //println!("{:?} ; {}",subgraph.nodes,monte_carlo(&subgraph,&weights,&mut rng,mc_tests));
                to_print.push_str(&format!("{:?} ; {} ; {} \n",cycle,monte_carlo(&subgraph,&weights,&mut rng,mc_tests),calculate_chance(&cycle,&weights)));
            }
            if !to_print.is_empty() {print!("{}",to_print)}
        }
        i = i + 1;
        graph.remove_node();
    } 
}

fn main() {
    // READING THE COMMAND LINE ARGUMENTS USING CLAP
    let matches = App::new("Cycle Finder Program")
                      .version("0.1")
                      .author("Jan Pel")
                      .about("Finds Cycles in weighted graph, also rates them")
                      .arg(Arg::with_name("cycles")
                           .short("c")
                           .long("cycles")
                           .help("Sets a custom config file")
                           .required(true)
                           .takes_value(true))
                      .arg(Arg::with_name("threads")
                           .short("t")
                           .help("sets the number of threads to use")
                           .takes_value(true))
                      .arg(Arg::with_name("montecarlo")
                           .short("mc")
                           .help("sets number of montecarlo simulations to run per cycle")
                           .takes_value(true))
                      .subcommand(SubCommand::with_name("test")
                                  .about("controls testing features")
                                  .version("1.3")
                                  .author("Someone E. <someone_else@other.com>")
                                  .arg(Arg::with_name("debug")
                                      .short("d")
                                      .help("print debug information verbosely")))
                          .get_matches();
    let n_workers :usize = matches.value_of("threads").unwrap_or("1").parse().unwrap();
    let number_mc_tests :usize = matches.value_of("montecarlo").unwrap_or("0").parse().unwrap(); 
    let max_cycle :usize = matches.value_of("cycles").unwrap().parse().unwrap();
    
    // The Bufreader is used to read in the graph, by being passed along to the reading functions
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
            cycle_worker(i,n_workers,nodes_temp ,edges_temp,max_cycle,number_mc_tests);
        });
        handles.push(handle);
    }
    // here we wait for all the threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
} 
