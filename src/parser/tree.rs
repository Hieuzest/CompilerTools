pub trait TreeNode : Sized {
    type Data;
    fn data(&self) -> &Self::Data;
    fn data_mut(&mut self) -> &mut Self::Data;
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&Self>;
    fn get_mut(&mut self, index: usize) -> Option<&mut Self>;
    fn remove(&mut self, index: usize) -> Self;
    fn swap_remove(&mut self, index: usize) -> Self;
    fn push(&mut self, rhs: Self);
    fn swap(&mut self, a: usize, b: usize);

    fn zipper(self) -> NodeZipper<Self> {
        NodeZipper { node: self, parent: None, index_in_parent: 0 }
    }
}


#[derive(Debug)]
pub struct NodeZipper<T: TreeNode> {
    pub node: T,
    parent: Option<Box<NodeZipper<T>>>,
    pub index_in_parent: usize,
}

impl<T: TreeNode> NodeZipper<T> {
    pub fn child(mut self, index: usize) -> Self {
        let child = self.node.swap_remove(index);

        NodeZipper {
            node: child,
            parent: Some(Box::new(self)),
            index_in_parent: index,
        }
    }

    pub fn check_parent(&self) -> bool {
        if let Some(_) = self.parent { true } else { false }
    }

    pub fn parent(self) -> Self {
        // Destructure this NodeZipper
        let NodeZipper { node, parent, index_in_parent } = self;

        // Destructure the parent NodeZipper
        let NodeZipper {
            node: mut parent_node,
            parent: parent_parent,
            index_in_parent: parent_index_in_parent,
        } = *parent.unwrap();


        parent_node.push(node);
        let len = parent_node.len();
        parent_node.swap(index_in_parent, len - 1);

        NodeZipper {
            node: parent_node,
            parent: parent_parent,
            index_in_parent: parent_index_in_parent,
        }
    }

    pub fn finish(mut self) -> T {
        while let Some(_) = self.parent {
            self = self.parent();
        }
        self.node
    }
}
