use std::fs::File;
use std::io::{BufRead, BufReader, Error, ErrorKind, Read,Write};
use std::collections::HashMap;
use std::io::prelude::*;
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

fn read_edges<R: std::io::BufRead>(io:&mut R,m:usize,n:usize) -> Result<Vec<Vec<usize>>,Error> {
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
            println!("{:?}",likes.pop());
        }
    }
    graph.remove(&node);    
}
fn find_cycles<W: std::io::Write>(mut graph:&mut HashMap<usize,Vec<usize>>,startnode : usize, out: &mut W,n:usize)   {
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
        }
        else if stack[l - 1].1.len() == 0 {
            stack.pop();
            path.pop();
        }
    } 
}

fn main() {
    // --snip--
    let filename = "graph_struct";
    let mut f = BufReader::new(File::open(filename).expect("file not found"));
    let options = read_number(&mut f,4).unwrap();
    let number_of_nodes = options[0];
    let number_of_edges = options[1];
    let mut nodes = read_number(&mut f,number_of_nodes).unwrap();
    let mut edges = read_edges(&mut f,number_of_edges,3).unwrap();
        
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
    let mut file = File::create("cycles.txt").unwrap();
    for n in nodes {
        find_cycles(&mut graph,n,&mut file,3);
        remove_edges(&mut graph, n);
    } 

}
