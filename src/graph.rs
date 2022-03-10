use crate::renderer::*;

pub trait Node {
    fn execute(&self, renderer: &mut Renderer);
}
pub struct DrawNode {}

impl Node for DrawNode {
    fn execute(&self, renderer: &mut Renderer) {
    }
}

pub struct PresentNode {}

impl Node for PresentNode {
    fn execute(&self, renderer: &mut Renderer) {
    }
}

pub struct Graph {
    pub nodes: Vec<Box<dyn Node>>,
    pub edges: Vec<(usize, usize)>,
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

    fn sort(&self) -> Vec<usize> {
        let mut local_edges = self.edges.clone();
        let mut exec_order = Vec::<usize>::new();
        let mut entry_nodes = Vec::<usize>::new();

        for (id, _node) in self.nodes.iter().enumerate() {
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

    pub fn execute(&mut self, renderer: &mut Renderer) {
        let exec_order = self.sort();

        for id in exec_order {
            let node = &self.nodes[id];
            node.execute(renderer);
        }
    }
}