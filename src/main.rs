trait Node {
    fn execute(&self);
}

struct DrawNode {}

impl Node for DrawNode {
    fn execute(&self) {
        println!("executing draw");
    }
}

struct PresentNode {}

impl Node for PresentNode {
    fn execute(&self) {
        println!("executing present");
    }
}

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
    fn sort(&self, graph: &Graph) -> Vec<usize> {
        let mut local_edges = graph.edges.clone();
        let mut exec_order = Vec::<usize>::new();
        let mut entry_nodes = Vec::<usize>::new();

        for (id, _node) in graph.nodes.iter().enumerate() {
            let num_deps = local_edges.iter().filter(|e| e.1 == id).count();
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
                .map(|e| e.1).collect::<Vec<usize>>();
            for dep_id in depending_nodes {
                local_edges.retain(|e| e.1 != dep_id);
                let num_deps = local_edges.iter().filter(|e| e.1 == dep_id).count();
                if num_deps == 0 {
                    entry_nodes.push(dep_id);
                }
            }
        }

        exec_order
    }

    pub fn execute(&self, graph: &Graph) {
        let exec_order = self.sort(graph);

        for id in exec_order {
            let node = &graph.nodes[id];
            node.execute();
        }
    }
}

fn main() {
    let renderer = Renderer {};

    let mut graph = Graph::new();

    let node1 = Box::new(DrawNode {});
    let node2 = Box::new(PresentNode {});

    let node1_id = graph.add(node1, &[]);
    let _node2_id = graph.add(node2, &[node1_id]);

    renderer.execute(&graph);
}
