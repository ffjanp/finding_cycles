This program reads in a graph from text file, and then prints all cycles smaller then length n in the graph to standard out.
The graph file needs to be structured as follows:

N (Number of nodes)
E (Number of edges)
node1 
...
node N
Edge1.begin Edge1.end
...
...
EdgeE.begin EdgeE.end

The program needs to be given the following arguments:
./../binary_file file_name max_cycle_lenth cycle_to_start cycle_to_end number_of_worker_threads

