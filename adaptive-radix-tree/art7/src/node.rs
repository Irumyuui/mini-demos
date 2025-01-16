use std::ptr::NonNull;

#[derive(Debug, Clone, Copy)]
pub(crate) enum NodePtr {
    Internal(NonNull<InternalNode>),
    Leaf(NonNull<LeafNode>),
    None,
}

impl NodePtr {
    pub(crate) fn new_leaf(key: Vec<u8>, value: Vec<u8>) -> Self {
        let leaf = Box::new(LeafNode { key, value });
        let ptr = Box::leak(leaf).into();
        Self::Leaf(ptr)
    }

    #[inline]
    pub(crate) fn find_min_leaf(&self) -> Option<NonNull<LeafNode>> {
        match self {
            NodePtr::Internal(n) => InternalNode::find_min_leaf(*n),
            NodePtr::Leaf(ptr) => Some(ptr.clone()),
            NodePtr::None => None,
        }
    }
}

pub(crate) struct LeafNode {
    pub(crate) key: Vec<u8>,
    pub(crate) value: Vec<u8>,
}

impl LeafNode {
    pub(crate) fn longest_common_prefix(&self, key: &[u8], start: usize) -> usize {
        let mut i = start;
        while i < self.key.len() && i < key.len() {
            if self.key[i] != key[i] {
                break;
            }
            i += 1;
        }
        i - start
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum NodeType {
    Node4,
    Node16,
    Node48,
    Node256,
}

#[repr(C)]
pub(crate) struct InternalNode {
    pub(crate) prefix: [u8; Self::MAX_PREFIX_SIZE],
    node_type: NodeType,
    num_children: u16,
    pub(crate) prefix_len: usize,
}

impl InternalNode {
    pub(crate) const MAX_PREFIX_SIZE: usize = 10;

    pub(crate) fn new(prefix: &[u8], node_type: NodeType) -> NonNull<Self> {
        assert!(prefix.len() <= Self::MAX_PREFIX_SIZE);
        let mut prefix_buf = [0; Self::MAX_PREFIX_SIZE];
        prefix_buf[..prefix.len()].copy_from_slice(prefix);

        let this = Self {
            prefix: prefix_buf,
            prefix_len: prefix.len(),
            node_type,
            num_children: 0,
        };

        match node_type {
            NodeType::Node4 => Node4::with_base(this).cast(),
            NodeType::Node16 => todo!(),
            NodeType::Node48 => todo!(),
            NodeType::Node256 => todo!(),
        }
    }

    pub(crate) fn prefix(&self) -> &[u8] {
        &self.prefix[..self.prefix_len.min(Self::MAX_PREFIX_SIZE)]
    }

    pub(crate) fn new_lazy_prefix(
        prefix: &[u8],
        prefix_len: usize,
        node_type: NodeType,
    ) -> NonNull<Self> {
        let mut this = Self::new(prefix, node_type);
        unsafe {
            this.as_mut().prefix_len = prefix_len;
        }
        this
    }

    pub(crate) fn split(&mut self, prefix_sp: usize) -> NonNull<InternalNode> {
        assert!(prefix_sp <= self.prefix_len.min(Self::MAX_PREFIX_SIZE) as usize);

        let new_node = InternalNode::new(&self.prefix[..prefix_sp], NodeType::Node4);
        let mut i = 0;
        let mut j = prefix_sp;
        while j < self.prefix_len as usize {
            self.prefix[i] = self.prefix[j];
            i += 1;
            j += 1;
        }
        self.prefix_len -= prefix_sp;

        new_node
    }

    pub(crate) fn is_full(&self) -> bool {
        todo!()
    }

    pub(crate) fn add_child(&mut self, byte: u8, child: NodePtr) {
        todo!()
    }

    pub(crate) fn grow(&mut self) -> NonNull<InternalNode> {
        todo!()
    }

    pub(crate) fn check_prefix(&self, key: &[u8]) -> usize {
        let mut i = 0;
        while i < self.prefix_len.min(Self::MAX_PREFIX_SIZE) as usize && i < key.len() {
            if self.prefix[i] != key[i] {
                break;
            }
            i += 1;
        }
        i
    }

    pub(crate) fn prefix_len(&self) -> usize {
        self.prefix_len as usize
    }

    pub(crate) unsafe fn get_child(&self, byte: u8) -> Option<&NodePtr> {
        todo!()
    }

    pub(crate) unsafe fn get_child_mut(&mut self, byte: u8) -> Option<&mut NodePtr> {
        todo!()
    }

    pub(crate) fn prefix_mismatch(node: NonNull<Self>, key: &[u8], depth: usize) -> usize {
        unsafe {
            let r = Self::MAX_PREFIX_SIZE
                .min(node.as_ref().prefix_len())
                .min(key.len() - depth);
            let mut i = 0;
            while i < r {
                if node.as_ref().prefix[i] != key[depth + i] {
                    return i;
                }
                i += 1;
            }

            if node.as_ref().prefix_len() > Self::MAX_PREFIX_SIZE {
                let leaf =
                    NodePtr::find_min_leaf(&NodePtr::Internal(node)).expect("must have leaf");

                let r = leaf.as_ref().key.len().min(key.len()) - depth;
                while i < r {
                    if leaf.as_ref().key[i] != key[depth + i] {
                        return i;
                    }
                    i += 1;
                }
            }

            i
        }
    }

    pub(crate) fn find_min_leaf(node: NonNull<Self>) -> Option<NonNull<LeafNode>> {
        todo!()
    }
}

pub(crate) struct Node4 {
    base: InternalNode,
    keys: [u8; 4],
    children: [NodePtr; 4],
}

impl Node4 {
    fn with_base(base: InternalNode) -> NonNull<Self> {
        let this = Self {
            base,
            keys: [0; 4],
            children: [NodePtr::None; 4],
        };
        NonNull::new(Box::into_raw(Box::new(this))).unwrap()
    }
}

pub(crate) struct Node16 {
    base: InternalNode,
    keys: [u8; 16],
    children: [NodePtr; 16],
}

pub(crate) struct Node48 {
    base: InternalNode,
    keys: [u8; 256],
    children: [NodePtr; 48],
}

pub(crate) struct Node256 {
    base: InternalNode,
    children: [NodePtr; 256],
}
