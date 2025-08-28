mod dynvec;
mod generational;
mod weak;

use dynvec::{DynVec, Handle};
use weak::Elem;

#[derive(Debug, Clone)]
struct TreeNode {
    name: String,
    parent: Option<Handle>,
    children: Vec<Handle>,
}

fn main() {
    println!("=== Generational References Demo ===\n");

    // Example 1: Self-referencing tree structure
    tree_example();

    // Example 2: Graph with node references
    graph_example();

    // Example 3: Content mutation invalidating references
    content_mutation_example();
}

fn tree_example() {
    println!("1. Tree Structure with Generational References");
    println!("==============================================");

    let mut nodes = DynVec::new();

    // Create root node
    let root_handle = nodes.insert(TreeNode {
        name: "Root".to_string(),
        parent: None,
        children: vec![],
    });

    // Create child nodes
    let child1_handle = nodes.insert(TreeNode {
        name: "Child1".to_string(),
        parent: Some(root_handle),
        children: vec![],
    });

    let child2_handle = nodes.insert(TreeNode {
        name: "Child2".to_string(),
        parent: Some(root_handle),
        children: vec![],
    });

    // Update root to include children
    if let Some(root) = nodes.get_mut(root_handle) {
        root.children.push(child1_handle);
        root.children.push(child2_handle);
    }

    // Demonstrate safe access through weak references
    let root_elem = Elem::new(&nodes, root_handle).unwrap();
    println!("Root node: {}", root_elem.name);

    // Accessing parent from child nodes
    for child_handle in &root_elem.children {
        if let Some(child_elem) = Elem::new(&nodes, *child_handle) {
            // Get the TreeNode data to access parent field
            if let Some(child_node) = nodes.get(*child_handle) {
                if let Some(parent_handle) = child_node.parent {
                    if let Some(parent) = nodes.get(parent_handle) {
                        println!("  Child: {} (parent: {})", child_elem.name, parent.name);
                    }
                }
            }
        }
    }
    println!();
}

fn graph_example() {
    println!("2. Graph with Node References");
    println!("=============================");

    let mut nodes = DynVec::new();

    // Create nodes
    let node_a = nodes.insert("Node A".to_string());
    let node_b = nodes.insert("Node B".to_string());
    let node_c = nodes.insert("Node C".to_string());

    // Create edges (stored as adjacency lists)
    let mut edges = DynVec::new();
    let _edge_ab = edges.insert(vec![node_a, node_b]);
    let _edge_bc = edges.insert(vec![node_b, node_c]);

    println!(
        "Created graph with {} nodes and {} edges",
        nodes.len(),
        edges.len()
    );

    // Access nodes safely
    if let Some(name) = nodes.get(node_a) {
        println!("Node A: {}", name);
    }

    println!();
}

fn content_mutation_example() {
    println!("3. Content Mutation Invalidating References");
    println!("===========================================");

    let mut nodes = DynVec::new();

    // Create a node
    let handle = nodes.insert("Original Content".to_string());
    println!("Created node with content: {}", nodes.get(handle).unwrap());

    // Mutate the content - this should still work since we don't bump generation
    *nodes.get_mut(handle).unwrap() = "Modified Content".to_string();
    println!("Content modified to: {}", nodes.get(handle).unwrap());

    // Create a weak reference after mutation
    let weak_ref = Elem::new(&nodes, handle).unwrap();
    println!("Weak reference created: {}", *weak_ref);

    // Drop the weak reference before mutating
    drop(weak_ref);

    // Now replace the content (which bumps generation)
    println!("Handle generation before replace: {}", handle.generation);
    let new_handle = nodes
        .replace(handle, "Replaced Content".to_string())
        .unwrap();
    println!(
        "Content replaced (generation bumped): {}",
        nodes.get(new_handle).unwrap()
    );
    println!("New handle generation: {}", new_handle.generation);

    // Try to create a weak reference with the old handle - should be invalid
    if let Some(old_weak_ref) = Elem::new(&nodes, handle) {
        println!("Old weak reference still valid: {}", *old_weak_ref);
    } else {
        println!("Old weak reference is invalid (as expected after generation bump)");
    }

    // Create a weak reference with the new handle - should work
    if let Some(new_weak_ref) = Elem::new(&nodes, new_handle) {
        println!("New weak reference is valid: {}", *new_weak_ref);
    }

    println!();
}
