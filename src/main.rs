use std::fs::File;
use std::io::{ BufReader,Error, BufWriter,stdout};
use std::collections::HashMap;
use std::env;
use std::thread;


fn read_number<R: std::io::BufRead>(io:&mut R,m:usize) -> Result<Vec<usize>,Error> {
    let mut v = vec![];
    for _ in 0..m {
    let mut line = String::new();
        io.read_line(&mut line)?;
        let n: usize = line.trim().parse::<usize>().unwrap(); v.push(n);
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

fn remove_edges(graph:&mut HashMap<usize,Vec<usize>>,node : usize) {
    for (_,likes) in &mut graph.iter_mut() {
        if likes.len() > 0 && likes[likes.len()-1] == node {
            likes.pop();
        }
    }
    graph.remove(&node);    
} 
fn find_cycles<W: std::io::Write>(graph:&mut HashMap<usize,Vec<usize>>,startnode : usize,n:usize,file:&mut W)   {
    let mut path = vec![startnode];
    let mut stack = vec![(startnode,graph[&startnode].to_vec())];
    while stack.len() > 0 {
        let l = stack.len();
        if stack[l - 1].1.len() > 0 {
            let nextnode = stack[l - 1].1.pop().unwrap();
            if nextnode == startnode {
//                out.write("{:?}",path);
                write!(file, "{:?} \n",path).unwrap();
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

fn cycle_worker(this_worker: usize,total_workers:usize) {
    // READING THE COMMAND LINE ARGUMENTS 
    let args: Vec<String> = env::args().collect();
    let filename = args[1].clone();
    let length :usize = args[2].parse().unwrap();
    let begin :usize = args[3].parse().unwrap();
    let end :usize = args[4].parse().unwrap();
    let mut f = BufReader::new(File::open(filename).expect("file not found"));

    // READING THE GRAPH FILE
    let graph_properties = read_number(&mut f,2).unwrap();
    let mut nodes = read_number(&mut f,graph_properties[0]).unwrap();
    let edges = read_edges(&mut f,graph_properties[1]).unwrap();
    
    //MAKING THE GRAPH, PER NODE THE EDGES OUT ARE STORED in  a sorted list
    let mut graph: HashMap<usize,Vec<usize>> = HashMap::new();
    for &node in &nodes {
        graph.insert(node,vec![]);
    }
    for edge in edges {
        graph.entry(edge[0]).or_insert(vec![]).push(edge[1]);
    }
    for (_,likes) in &mut graph {
        likes.sort_unstable();
    }
    nodes.reverse();

    // HERE THE NODES AND EDGES ARE REMOVED WHICH ARE SKIPPED
    for &n in &nodes[0..begin] {
        remove_edges(&mut graph,n);
    }
    // HERE ALL THE CYCLES OF LENGTH "length" STARTING AT THE NODES begin..end ARE FOUND AND 
    // PRINTED TO STANDARD OUT.
    let mut file = BufWriter::new(stdout()); 
    let mut i = 1;
    for &n in &nodes[begin..end] {
        if i % total_workers == this_worker { 
            find_cycles(&mut graph,n,length,&mut file);
        }
        i = i + 1;
        remove_edges(&mut graph, n);
    } 

}

fn main() {
    let n_workers = 1;
    let mut handles = vec![];
    for i in 0..n_workers {
        let handle = thread::spawn(move|| {
            cycle_worker(i,n_workers);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
} 
