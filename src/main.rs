use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read,Write};
use std::collections::HashMap;
use std::io::prelude::*;
use std::env;

fn read_number<R: std::io::BufRead>(io:&mut R,m:usize) -> Result<Vec<usize>,Error> {
    let mut v = vec![];
    for _ in 0..m {
    let mut line = String::new();
        io.read_line(&mut line)?;
        let n: usize = line.trim().parse::<usize>().unwrap();
        v.push(n);
    }
    Ok(v)
}

fn read_edges<R: std::io::BufRead>(io:&mut R,m:usize) -> Result<Vec<Vec<usize>>,Error> {
    let mut v = vec![];
    for _ in 0..m {
        let mut line = String::new();
        io.read_line(&mut line)?;
        let edge: Vec<usize> = line.split(' ').map(|x| x.trim().parse::<usize>().unwrap()).collect();
        v.push(edge);
    }
    Ok(v)
}

fn remove_edges(mut graph:&mut HashMap<usize,Vec<usize>>,node : usize) {
    for (_,likes) in &mut graph.iter_mut() {
        if likes.len() > 0 && likes[likes.len()-1] == node {
            likes.pop();
        }
    }
    graph.remove(&node);    
} fn find_cycles(mut graph:&mut HashMap<usize,Vec<usize>>,startnode : usize,n:usize)   {
    let mut path = vec![startnode];
    let mut stack = vec![(startnode,graph[&startnode].to_vec())];
    let mut nextnode :usize = 0;
    while stack.len() > 0 {
        let l = stack.len();
        if stack[l - 1].1.len() > 0 {
            nextnode = stack[l - 1].1.pop().unwrap();
            if nextnode == startnode {
//                out.write("{:?}",path);
                println!( "{:?}",path);
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
}

fn main() {
    // READING THE COMMAND LINE ARGUMENTS 
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let filename = args[1].to_string();
    let length :usize = args[2].parse().unwrap();
    let begin :usize = args[3].parse().unwrap();
    let end :usize = args[4].parse().unwrap();
    let mut f = BufReader::new(File::open(filename).expect("file not found"));

    // READING THE GRAPH FILE
    let graph_properties = read_number(&mut f,2).unwrap();
    let mut nodes = read_number(&mut f,graph_properties[0]).unwrap();
    let mut edges = read_edges(&mut f,graph_properties[1]).unwrap();
    
    //MAKING THE GRAPH, PER NODE THE EDGES OUT ARE STORED in  a sorted list
    let mut graph: HashMap<usize,Vec<usize>> = HashMap::new();
    for &node in &nodes {
        graph.insert(node,vec![]);
    }
    for edge in edges {
        graph.entry(edge[0]).or_insert(vec![]).push(edge[1]);
    }
    for (node,likes) in &mut graph {
        likes.sort_unstable();
    }
    nodes.reverse();

    // HERE THE NODES AND EDGES ARE REMOVED WHICH ARE SKIPPED
    for &n in &nodes[0..begin] {
        remove_edges(&mut graph,n);
    }
    // HERE ALL THE CYCLES OF LENGTH "length" STARTING AT THE NODES begin..end ARE FOUND AND 
    // PRINTED TO STANDARD OUT.
    for &n in &nodes[begin..end] {
        find_cycles(&mut graph,n,length);
        remove_edges(&mut graph, n);
    } 

}
