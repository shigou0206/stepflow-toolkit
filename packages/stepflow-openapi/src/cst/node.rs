use tree_sitter::TreeCursor;
use std::borrow::Cow;
use std::sync::Arc;

/// 遍历策略枚举
/// 
/// 定义了树遍历的不同顺序。注意：中序遍历仅适用于二叉树，
/// 对于通用 N 叉树没有明确定义，因此不包含在此枚举中。
#[derive(Debug, Clone, Copy)]
pub enum TraversalOrder {
    /// 前序遍历：父节点 -> 子节点
    PreOrder,
    /// 后序遍历：子节点 -> 父节点  
    PostOrder,
    /// 广度优先遍历：按层级遍历
    BreadthFirst,
}

/// 轻量级树遍历迭代器
/// 
/// 使用直接匹配而非 trait 对象，减少虚调用开销。
/// 支持零拷贝遍历，所有节点引用都指向原始树结构。
pub struct TreeIterator<'a> {
    order: TraversalOrder,
    stack: Vec<&'a TreeCursorSyntaxNode>,
    queue: std::collections::VecDeque<&'a TreeCursorSyntaxNode>,
    post_order_stack: Vec<(&'a TreeCursorSyntaxNode, usize)>,
}

impl<'a> TreeIterator<'a> {
    /// 创建前序遍历迭代器
    /// 
    /// # Example
    /// ```
    /// # use apidom_cst::{CstParser, TreeIterator};
    /// let cst = CstParser::parse(r#"{"key": "value"}"#);
    /// let iter = TreeIterator::new_preorder(&cst);
    /// for node in iter {
    ///     println!("Node: {}", node.kind);
    /// }
    /// ```
    pub fn new_preorder(root: &'a TreeCursorSyntaxNode) -> Self {
        TreeIterator {
            order: TraversalOrder::PreOrder,
            stack: vec![root],
            queue: std::collections::VecDeque::new(),
            post_order_stack: Vec::new(),
        }
    }
    
    /// 创建后序遍历迭代器
    /// 
    /// # Example  
    /// ```
    /// # use apidom_cst::{CstParser, TreeIterator};
    /// let cst = CstParser::parse(r#"{"key": "value"}"#);
    /// let iter = TreeIterator::new_postorder(&cst);
    /// for node in iter {
    ///     println!("Node: {}", node.kind);
    /// }
    /// ```
    pub fn new_postorder(root: &'a TreeCursorSyntaxNode) -> Self {
        TreeIterator {
            order: TraversalOrder::PostOrder,
            stack: Vec::new(),
            queue: std::collections::VecDeque::new(),
            post_order_stack: vec![(root, 0)],
        }
    }
    
    /// 创建广度优先遍历迭代器
    /// 
    /// # Example
    /// ```
    /// # use apidom_cst::{CstParser, TreeIterator};
    /// let cst = CstParser::parse(r#"{"key": "value"}"#);
    /// let iter = TreeIterator::new_breadth_first(&cst);
    /// for node in iter {
    ///     println!("Node: {}", node.kind);
    /// }
    /// ```
    pub fn new_breadth_first(root: &'a TreeCursorSyntaxNode) -> Self {
        TreeIterator {
            order: TraversalOrder::BreadthFirst,
            stack: Vec::new(),
            queue: std::collections::VecDeque::from([root]),
            post_order_stack: Vec::new(),
        }
    }
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = &'a TreeCursorSyntaxNode;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.order {
            TraversalOrder::PreOrder => {
                if let Some(node) = self.stack.pop() {
                    // 将子节点按逆序压入栈中（这样弹出时就是正序）
                    for child in node.children.iter().rev() {
                        self.stack.push(child);
                    }
                    Some(node)
                } else {
                    None
                }
            }
            TraversalOrder::PostOrder => {
                while let Some((node, child_idx)) = self.post_order_stack.pop() {
                    if child_idx >= node.children.len() {
                        return Some(node);
                    } else {
                        self.post_order_stack.push((node, child_idx + 1));
                        self.post_order_stack.push((&node.children[child_idx], 0));
                    }
                }
                None
            }
            TraversalOrder::BreadthFirst => {
                if let Some(node) = self.queue.pop_front() {
                    for child in &node.children {
                        self.queue.push_back(child);
                    }
                    Some(node)
                } else {
                    None
                }
            }
        }
    }
}

/// CST 节点包装器
/// 
/// 这是我们对 tree-sitter 节点的包装，提供了额外的功能：
/// - 零拷贝文本提取
/// - 共享源码引用（Arc<str>）
/// - 多种遍历方式
/// - 字段名记录
/// - 错误检测
/// - 内存优化（使用 Arc<str> 而非 Arc<Vec<u8>>）
#[derive(Debug, Clone)]
pub struct TreeCursorSyntaxNode {
    /// 节点类型（如 "object", "array", "string" 等）
    pub kind: String,
    /// 节点在源码中的起始字节位置
    pub start_byte: usize,
    /// 节点在源码中的结束字节位置
    pub end_byte: usize,
    /// 节点在源码中的起始位置（行列）
    pub start_point: tree_sitter::Point,
    /// 节点在源码中的结束位置（行列）
    pub end_point: tree_sitter::Point,
    /// 是否为命名节点（非符号节点）
    pub named: bool,
    /// 是否包含语法错误
    pub error: bool,
    /// 字段名（如果此节点是某个字段的值）
    pub field_name: Option<String>,
    
    /// 使用 Arc<str> 共享源码，比 Arc<Vec<u8>> 更高效
    /// 这样可以直接进行字符串切片操作，避免 UTF-8 转换
    source: Arc<str>,
    
    /// 子节点列表
    /// 未来可以考虑实现懒加载，仅在首次访问时构造
    pub children: Vec<TreeCursorSyntaxNode>,
}

impl TreeCursorSyntaxNode {
    /// 从 TreeCursor 构造节点（创建新的 Arc 源码）
    /// 
    /// # Arguments
    /// * `cursor` - tree-sitter 游标
    /// * `src` - 源码字节数组
    /// 
    /// # Returns
    /// 新构造的 CST 节点
    pub fn from_cursor(cursor: &TreeCursor, src: &[u8]) -> Self {
        let source_str = String::from_utf8_lossy(src);
        let shared_source = Arc::from(source_str.as_ref());
        Self::from_cursor_with_shared_source(cursor, shared_source)
    }
    
    /// 从已有的 Arc 源码构造节点（用于子节点，避免重复克隆）
    /// 
    /// # Arguments
    /// * `cursor` - tree-sitter 游标
    /// * `source` - 共享的源码引用
    /// 
    /// # Returns
    /// 新构造的 CST 节点，与父节点共享源码
    pub fn from_cursor_with_shared_source(cursor: &TreeCursor, source: Arc<str>) -> Self {
        Self::from_cursor_with_shared_source_and_field(cursor, source, None)
    }
    
    pub fn from_cursor_with_shared_source_and_field(cursor: &TreeCursor, source: Arc<str>, field_name: Option<String>) -> Self {
        let node = cursor.node();
        let kind = node.kind().to_string();
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let start_point = node.start_position();
        let end_point = node.end_position();
        let named = node.is_named();
        let error = node.has_error();

        TreeCursorSyntaxNode {
            kind,
            start_byte: start_byte as usize,
            end_byte: end_byte as usize,
            start_point,
            end_point,
            named,
            error,
            field_name,
            source,
            children: Vec::new(),
        }
    }
    
    /// 延迟获取节点的文本内容
    /// 
    /// 使用 Arc<str> 的优势：可以直接进行字符串切片，
    /// 无需 UTF-8 转换，实现真正的零拷贝。
    /// 
    /// # Returns
    /// 节点对应的文本内容
    /// 
    /// # Example
    /// ```
    /// # use apidom_cst::CstParser;
    /// let cst = CstParser::parse(r#"{"key": "value"}"#);
    /// let text = cst.text();
    /// println!("Node text: {}", text);
    /// ```
    pub fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.source[self.start_byte..self.end_byte])
    }
    
    /// 检查节点是否包含语法错误
    /// 
    /// # Returns
    /// 如果节点或其子树中有错误则返回 true
    pub fn has_error(&self) -> bool {
        self.error
    }
    
    /// 获取节点的字段名
    /// 
    /// 在 JSON 中，对象的键值对中，键和值都有对应的字段名。
    /// 
    /// # Returns
    /// 字段名的字符串引用，如果没有字段名则返回 None
    pub fn field_name(&self) -> Option<&str> {
        self.field_name.as_deref()
    }
    
    /// 创建前序遍历迭代器（零拷贝）
    /// 
    /// 前序遍历会先访问父节点，然后按顺序访问所有子节点。
    /// 
    /// # Returns
    /// 前序遍历迭代器
    /// 
    /// # Example
    /// ```
    /// # use apidom_cst::CstParser;
    /// let cst = CstParser::parse(r#"{"key": "value"}"#);
    /// for node in cst.iter_preorder() {
    ///     println!("Visiting: {}", node.kind);
    /// }
    /// ```
    pub fn iter_preorder(&self) -> TreeIterator {
        TreeIterator::new_preorder(self)
    }
    
    /// 创建后序遍历迭代器（零拷贝）
    /// 
    /// 后序遍历会先访问所有子节点，最后访问父节点。
    /// 
    /// # Returns
    /// 后序遍历迭代器
    pub fn iter_postorder(&self) -> TreeIterator {
        TreeIterator::new_postorder(self)
    }
    
    /// 创建广度优先遍历迭代器（零拷贝）
    /// 
    /// 广度优先遍历会按层级顺序访问节点。
    /// 
    /// # Returns
    /// 广度优先遍历迭代器
    pub fn iter_breadth_first(&self) -> TreeIterator {
        TreeIterator::new_breadth_first(self)
    }
    
    /// 按深度优先搜索查找特定类型的节点
    /// 
    /// # Arguments
    /// * `kind` - 要查找的节点类型
    /// 
    /// # Returns
    /// 匹配类型的所有节点引用
    /// 
    /// # Example
    /// ```
    /// # use apidom_cst::CstParser;
    /// let cst = CstParser::parse(r#"{"key": "value"}"#);
    /// let strings = cst.find_nodes_by_kind("string");
    /// for string_node in strings {
    ///     println!("Found string: {}", string_node.text());
    /// }
    /// ```
    pub fn find_nodes_by_kind(&self, kind: &str) -> Vec<&TreeCursorSyntaxNode> {
        let mut result = Vec::new();
        self.collect_nodes_by_kind(kind, &mut result);
        result
    }
    
    /// 递归收集指定类型的节点（内部辅助方法）
    fn collect_nodes_by_kind<'a>(&'a self, kind: &str, result: &mut Vec<&'a TreeCursorSyntaxNode>) {
        if self.kind == kind {
            result.push(self);
        }
        for child in &self.children {
            child.collect_nodes_by_kind(kind, result);
        }
    }
    
    /// 获取共享的源码引用
    /// 
    /// 用于检查内存使用情况和引用计数。
    /// Arc<str> 比 Arc<Vec<u8>> 更高效，因为它避免了 UTF-8 验证开销。
    /// 
    /// # Returns
    /// Arc 包装的源码引用
    pub fn shared_source(&self) -> &Arc<str> {
        &self.source
    }
}