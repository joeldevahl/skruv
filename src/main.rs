trait Node {}

struct BasicNode {}

impl Node for BasicNode {}

struct Graph {
    nodes: Vec<Box<dyn Node>>,
    edges: Vec<(usize, usize)>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add(&mut self, node: Box<dyn Node>, deps: &[usize]) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        for dep in deps {
            // TODO: check for loops
            self.edges.push((*dep, id));
        }
        id
    }
}

struct Renderer {}

impl Renderer {
    pub fn execute(&mut self, graph: &Graph) {
        /* From wikipedia:
        L ← Empty list that will contain the sorted elements
        S ← Set of all nodes with no incoming edge

        while S is not empty do
            remove a node n from S
            add n to L
            for each node m with an edge e from n to m do
                remove edge e from the graph
                if m has no other incoming edges then
                    insert m into S

        if graph has edges then
            return error   (graph has at least one cycle)
        else
            return L   (a topologically sorted order)
        */

        let local_edges = graph.edges.clone();
        let mut exec_order = Vec::<usize>::new();
        let mut entry_nodes = Vec::<usize>::new();

        for (id, _node) in graph.nodes.iter().enumerate() {
            let num_deps = local_edges.iter().filter(|e| e.0 == id).count();
            if num_deps == 0 {
                entry_nodes.push(id);
            }
        }

        while !entry_nodes.is_empty() {
            let id = entry_nodes.pop().unwrap();
            exec_order.push(id);

            let depending_nodes = local_edges
                .iter()
                .filter(|e| e.0 == id)
                .map(|e| e.1)
                .collect();
            for dep_id in depending_nodes {
                local_edges.retain(|e| e.1 != dep_id);
                let num_deps = local_edges.iter().filter(|e| e.0 == dep_id).count();
            }
        }

        for id in exec_order {
            let _node = &graph.nodes[id];
            println!("got node {:?}", id);
        }
    }
}

fn main() {
    let mut renderer = Renderer {};

    let mut graph = Graph::new();

    let node1 = Box::new(BasicNode {});
    let node2 = Box::new(BasicNode {});

    let node1_id = graph.add(node1, &[]);
    let _node2_id = graph.add(node2, &[node1_id]);

    renderer.execute(&mut graph);
}
