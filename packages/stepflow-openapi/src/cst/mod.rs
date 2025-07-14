mod node;
pub use node::{TreeCursorSyntaxNode, TreeIterator, TraversalOrder};

use tree_sitter::{Parser, TreeCursor, Language};
use std::cell::RefCell;
use std::sync::Arc;

/// 支持的源码类型
/// 
/// 定义了 CST 解析器可以处理的不同格式类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    /// JSON 格式
    Json,
    /// YAML 格式
    Yaml,
}

impl SourceType {
    /// 从文件扩展名推断源码类型
    /// 
    /// # Arguments
    /// * `extension` - 文件扩展名（如 "json", "yaml", "yml"）
    /// 
    /// # Returns
    /// 对应的源码类型，如果无法识别则返回 None
    /// 
    /// # Example
    /// ```
    /// use apidom_cst::SourceType;
    /// 
    /// assert_eq!(SourceType::from_extension("json"), Some(SourceType::Json));
    /// assert_eq!(SourceType::from_extension("yaml"), Some(SourceType::Yaml));
    /// assert_eq!(SourceType::from_extension("yml"), Some(SourceType::Yaml));
    /// assert_eq!(SourceType::from_extension("txt"), None);
    /// ```
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "json" => Some(SourceType::Json),
            "yaml" | "yml" => Some(SourceType::Yaml),
            _ => None,
        }
    }
    
    /// 获取源码类型的显示名称
    /// 
    /// # Returns
    /// 源码类型的字符串表示
    pub fn display_name(&self) -> &'static str {
        match self {
            SourceType::Json => "JSON",
            SourceType::Yaml => "YAML",
        }
    }

    /// 使用启发式方法检测源码类型
    /// 
    /// 基于内容特征来推断最可能的格式类型。
    /// 
    /// # Arguments
    /// * `source` - 要检测的源码字符串
    /// 
    /// # Returns
    /// 推荐的源码类型
    pub fn detect_from_content(source: &str) -> Self {
        let source = source.trim();
        
        // 明显的 JSON 特征
        if source.starts_with('{') || source.starts_with('[') {
            return SourceType::Json;
        }
        
        // YAML 文档分隔符
        if source.starts_with("---") {
            return SourceType::Yaml;
        }
        
        // 检查是否有 YAML 风格的键值对（key: value 且不在引号内）
        let lines: Vec<&str> = source.lines().collect();
        let mut yaml_indicators = 0;
        let mut json_indicators = 0;
        
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue; // 跳过空行和注释
            }
            
            // YAML 风格的键值对
            if trimmed.contains(':') && !trimmed.starts_with('"') && !trimmed.starts_with('{') {
                yaml_indicators += 1;
            }
            
            // YAML 列表项
            if trimmed.starts_with("- ") {
                yaml_indicators += 1;
            }
            
            // JSON 风格的引号键
            if trimmed.contains(r#"":"#) {
                json_indicators += 1;
            }
        }
        
        if yaml_indicators > json_indicators {
            SourceType::Yaml
        } else {
            SourceType::Json
        }
    }
}

// 线程本地 Parser，避免多线程竞争
// 
// 每个线程都有自己的 Parser 实例，避免了全局锁的开销。
// 分别维护 JSON 和 YAML 的解析器实例。
thread_local! {
    static THREAD_LOCAL_JSON_PARSER: RefCell<Option<Parser>> = RefCell::new(None);
    static THREAD_LOCAL_YAML_PARSER: RefCell<Option<Parser>> = RefCell::new(None);
}

/// 通用的 Parser 操作辅助函数
/// 
/// 自动获取对应类型的线程本地 Parser，执行操作，然后归还。
/// 这避免了重复的 get/return 样板代码。
/// 
/// # Arguments
/// * `source_type` - 源码类型
/// * `f` - 要执行的操作，接收可变 Parser 引用
/// 
/// # Returns
/// 操作的返回值
fn with_parser<F, R>(source_type: SourceType, f: F) -> R 
where 
    F: FnOnce(&mut Parser) -> R 
{
    match source_type {
        SourceType::Json => {
            THREAD_LOCAL_JSON_PARSER.with(|parser_cell| {
                let mut parser_opt = parser_cell.borrow_mut();
                if parser_opt.is_none() {
                    let mut parser = Parser::new();
                    let language = tree_sitter_json::LANGUAGE;
                    parser.set_language(&Language::new(language)).unwrap();
                    *parser_opt = Some(parser);
                }
                
                // 取出 parser，执行操作，然后放回
                let mut parser = parser_opt.take().unwrap();
                let result = f(&mut parser);
                *parser_opt = Some(parser);
                result
            })
        }
        SourceType::Yaml => {
            THREAD_LOCAL_YAML_PARSER.with(|parser_cell| {
                let mut parser_opt = parser_cell.borrow_mut();
                if parser_opt.is_none() {
                    let mut parser = Parser::new();
                    let language = tree_sitter_yaml::LANGUAGE;
                    parser.set_language(&Language::new(language)).unwrap();
                    *parser_opt = Some(parser);
                }
                
                // 取出 parser，执行操作，然后放回
                let mut parser = parser_opt.take().unwrap();
                let result = f(&mut parser);
                *parser_opt = Some(parser);
                result
            })
        }
    }
}

/// 递归遍历并构造 CST 子节点的通用函数
/// 
/// 这个函数被提取出来避免在多个地方重复相同的逻辑。
/// 
/// # Arguments
/// * `cursor` - tree-sitter 游标
/// * `shared_source` - 共享的源码引用
/// * `parent` - 父节点，将填充其 children
fn descend_and_build_children(
    cursor: &mut TreeCursor,
    shared_source: &Arc<str>,
    parent: &mut TreeCursorSyntaxNode
) {
    if cursor.goto_first_child() {
        // We need to get the parent node before moving to the first child
        cursor.goto_parent();
        let parent_node = cursor.node(); // Get the parent node for field name lookup
        cursor.goto_first_child(); // Move back to first child
        
        let mut child_index = 0u32;
        
        loop {
            // Get field name for this child from parent
            let field_name = parent_node.field_name_for_child(child_index);
            
            let mut child = TreeCursorSyntaxNode::from_cursor_with_shared_source_and_field(
                cursor, 
                shared_source.clone(),
                field_name.map(|s| s.to_string())
            );
            descend_and_build_children(cursor, shared_source, &mut child);
            parent.children.push(child);
            
            child_index += 1;
            if !cursor.goto_next_sibling() { 
                break;
            }
        }
        cursor.goto_parent();
    }
}

/// CST 解析器构建器
/// 
/// 提供流畅的 API 来解析 JSON 和 YAML 并进行各种操作。
/// 支持自动类型检测和手动指定源码类型。
/// 
/// # Example
/// ```
/// use apidom_cst::{CstParser, SourceType};
/// 
/// // 自动检测（默认 JSON）
/// let cst = CstParser::parse(r#"{"key": "value"}"#);
/// 
/// // 明确指定类型
/// let json_cst = CstParser::parse_as(r#"{"key": "value"}"#, SourceType::Json);
/// let yaml_cst = CstParser::parse_as("key: value", SourceType::Yaml);
/// 
/// // 链式调用
/// let strings: Vec<_> = CstParser::parse_as("key: value", SourceType::Yaml)
///     .find_nodes_by_kind("flow_entry")
///     .into_iter()
///     .map(|node| node.text())
///     .collect();
/// ```
pub struct CstParser;

impl CstParser {
    /// 解析源码为 CST（默认使用 JSON 格式）
    /// 
    /// 这是向后兼容的方法，默认将输入视为 JSON。
    /// 如果需要解析其他格式，请使用 `parse_as` 方法。
    /// 
    /// # Arguments
    /// * `source` - 要解析的源码字符串
    /// 
    /// # Returns
    /// 解析后的 CST 根节点
    /// 
    /// # Example
    /// ```
    /// use apidom_cst::CstParser;
    /// let cst = CstParser::parse(r#"{"name": "example"}"#);
    /// println!("Root kind: {}", cst.kind);
    /// ```
    pub fn parse(source: &str) -> TreeCursorSyntaxNode {
        Self::parse_as(source, SourceType::Json)
    }
    
    /// 解析指定类型的源码为 CST
    /// 
    /// 这是主要的入口点，将源码解析为具体语法树。
    /// 使用线程本地 Parser 以获得最佳性能。
    /// 
    /// # Arguments
    /// * `source` - 要解析的源码字符串
    /// * `source_type` - 源码类型
    /// 
    /// # Returns
    /// 解析后的 CST 根节点
    /// 
    /// # Example
    /// ```
    /// use apidom_cst::{CstParser, SourceType};
    /// 
    /// // 解析 JSON
    /// let json_cst = CstParser::parse_as(r#"{"name": "example"}"#, SourceType::Json);
    /// 
    /// // 解析 YAML
    /// let yaml_cst = CstParser::parse_as("name: example", SourceType::Yaml);
    /// ```
    pub fn parse_as(source: &str, source_type: SourceType) -> TreeCursorSyntaxNode {
        with_parser(source_type, |parser| {
            // 解析源码得到 Tree
            let tree = parser.parse(source, None)
                .unwrap_or_else(|| panic!("Failed to parse {} source", source_type.display_name()));

            // 用 cursor 从根节点构造我们的包装
            let mut cursor = tree.walk();
            let shared_source: Arc<str> = Arc::from(source);
            let mut root = TreeCursorSyntaxNode::from_cursor_with_shared_source(&cursor, shared_source.clone());

            // 递归遍历所有子节点，填充 children
            descend_and_build_children(&mut cursor, &shared_source, &mut root);

            root
        })
    }
    
    /// 智能解析：尝试自动检测源码类型
    /// 
    /// 使用启发式方法检测最可能的格式，然后尝试解析。
    /// 如果检测错误，会尝试另一种格式。
    /// 
    /// # Arguments
    /// * `source` - 要解析的源码字符串
    /// 
    /// # Returns
    /// 解析后的 CST 根节点和检测到的源码类型
    /// 
    /// # Example
    /// ```
    /// use apidom_cst::CstParser;
    /// 
    /// let (cst, detected_type) = CstParser::parse_smart(r#"{"key": "value"}"#);
    /// println!("Detected type: {}", detected_type.display_name());
    /// ```
    pub fn parse_smart(source: &str) -> (TreeCursorSyntaxNode, SourceType) {
        // 使用启发式方法检测格式
        let detected_type = SourceType::detect_from_content(source);
        
        // 首先尝试检测到的格式
        if let Ok(tree) = Self::try_parse_as(source, detected_type) {
            return (tree, detected_type);
        }
        
        // 如果失败，尝试另一种格式
        let fallback_type = match detected_type {
            SourceType::Json => SourceType::Yaml,
            SourceType::Yaml => SourceType::Json,
        };
        
        let tree = Self::parse_as(source, fallback_type);
        (tree, fallback_type)
    }
    
    /// 尝试解析指定类型的源码，不抛出异常
    /// 
    /// # Arguments
    /// * `source` - 要解析的源码字符串
    /// * `source_type` - 源码类型
    /// 
    /// # Returns
    /// 解析成功返回 Ok(CST)，失败返回 Err
    fn try_parse_as(source: &str, source_type: SourceType) -> Result<TreeCursorSyntaxNode, String> {
        with_parser(source_type, |parser| {
            let tree = match parser.parse(source, None) {
                Some(tree) => tree,
                None => {
                    return Err(format!("Failed to parse {} source", source_type.display_name()));
                }
            };
            
            // 检查是否有语法错误
            let root_node = tree.root_node();
            if root_node.has_error() {
                return Err(format!("{} source has syntax errors", source_type.display_name()));
            }
            
            // 构造 CST
            let mut cursor = tree.walk();
            let shared_source: Arc<str> = Arc::from(source);
            let mut root = TreeCursorSyntaxNode::from_cursor_with_shared_source(&cursor, shared_source.clone());

            descend_and_build_children(&mut cursor, &shared_source, &mut root);

            Ok(root)
        })
    }
}

/// 便利函数：把一整段 JSON 源码变成我们的 `TreeCursorSyntaxNode` 树
/// 
/// 这是 `CstParser::parse` 的别名，为了向后兼容保留。
/// 推荐使用 `CstParser::parse` 或 `CstParser::parse_as`。
/// 
/// # Arguments
/// * `source` - JSON 源码字符串
/// 
/// # Returns
/// 解析后的 CST 根节点
pub fn parse_json_to_cst(source: &str) -> TreeCursorSyntaxNode {
    CstParser::parse(source)
}

/// 扩展 TreeCursorSyntaxNode 以支持构建器模式
impl TreeCursorSyntaxNode {
    /// 创建前序遍历迭代器的构建器方法
    /// 
    /// # Returns
    /// 前序遍历迭代器
    /// 
    /// # Example
    /// ```
    /// use apidom_cst::CstParser;
    /// let cst = CstParser::parse(r#"{"key": "value"}"#);
    /// let nodes: Vec<_> = cst.preorder().collect();
    /// ```
    pub fn preorder(&self) -> TreeIterator {
        self.iter_preorder()
    }
    
    /// 创建后序遍历迭代器的构建器方法
    /// 
    /// # Returns
    /// 后序遍历迭代器
    pub fn postorder(&self) -> TreeIterator {
        self.iter_postorder()
    }
    
    /// 创建广度优先遍历迭代器的构建器方法
    /// 
    /// # Returns
    /// 广度优先遍历迭代器
    pub fn breadth_first(&self) -> TreeIterator {
        self.iter_breadth_first()
    }
}

/// 示例：展示如何使用新的 CST 功能
/// 
/// 这个函数演示了 CST 解析器的各种功能，包括：
/// - 多格式解析（JSON 和 YAML）
/// - 智能格式检测
/// - 基本解析和错误检测
/// - 延迟文本提取
/// - 字段名记录
/// - 多种遍历方式
/// - 节点查找
/// - 内存优化
/// 
/// # Arguments
/// * `source` - 要演示的源码字符串
/// * `source_type` - 可选的源码类型，如果不指定则使用智能检测
pub fn demonstrate_cst_features_multi_format(source: &str, source_type: Option<SourceType>) {
    println!("=== 多格式 CST 功能演示 ===");
    println!("输入源码: {}", source);
    
    // 1. 解析源码到 CST
    let (cst, detected_type) = match source_type {
        Some(st) => {
            println!("指定格式: {}", st.display_name());
            (CstParser::parse_as(source, st), st)
        }
        None => {
            println!("使用智能检测...");
            let (cst, detected) = CstParser::parse_smart(source);
            println!("检测到格式: {}", detected.display_name());
            (cst, detected)
        }
    };
    
    println!("\n1. 基本信息:");
    println!("   格式类型: {}", detected_type.display_name());
    println!("   根节点类型: {}", cst.kind);
    println!("   是否有错误: {}", cst.has_error());
    println!("   子节点数量: {}", cst.children.len());
    
    // 2. 演示延迟文本提取
    println!("\n2. 延迟文本提取:");
    println!("   根节点文本长度: {} 字节", cst.text().len());
    
    // 3. 演示格式特定的节点类型
    println!("\n3. 格式特定的节点类型:");
    match detected_type {
        SourceType::Json => {
            let objects = cst.find_nodes_by_kind("object");
            let arrays = cst.find_nodes_by_kind("array");
            let strings = cst.find_nodes_by_kind("string");
            let numbers = cst.find_nodes_by_kind("number");
            
            println!("   JSON 对象: {} 个", objects.len());
            println!("   JSON 数组: {} 个", arrays.len());
            println!("   字符串: {} 个", strings.len());
            println!("   数字: {} 个", numbers.len());
            
            // 显示字符串内容
            if !strings.is_empty() {
                println!("   字符串内容:");
                for (i, string) in strings.iter().take(3).enumerate() {
                    println!("     {}: {}", i + 1, string.text());
                }
            }
        }
        SourceType::Yaml => {
            let documents = cst.find_nodes_by_kind("document");
            let block_mappings = cst.find_nodes_by_kind("block_mapping");
            let block_sequences = cst.find_nodes_by_kind("block_sequence");
            let plain_scalars = cst.find_nodes_by_kind("plain_scalar");
            let quoted_scalars = cst.find_nodes_by_kind("double_quote_scalar");
            
            println!("   YAML 文档: {} 个", documents.len());
            println!("   块映射: {} 个", block_mappings.len());
            println!("   块序列: {} 个", block_sequences.len());
            println!("   普通标量: {} 个", plain_scalars.len());
            println!("   引用标量: {} 个", quoted_scalars.len());
            
            // 显示标量内容
            if !plain_scalars.is_empty() {
                println!("   标量内容:");
                for (i, scalar) in plain_scalars.iter().take(3).enumerate() {
                    println!("     {}: {}", i + 1, scalar.text());
                }
            }
        }
    }
    
    // 4. 演示构建器风格的迭代器遍历
    println!("\n4. 前序遍历前 10 个节点:");
    for (i, node) in cst.preorder().take(10).enumerate() {
        let error_mark = if node.has_error() { " [ERROR]" } else { "" };
        println!("   {}: {} ({}..{}){}", 
                 i + 1, node.kind, node.start_byte, node.end_byte, error_mark);
    }
    
    // 5. 演示字段名记录（主要用于 JSON）
    if detected_type == SourceType::Json {
        println!("\n5. JSON 字段名记录:");
        let pairs = cst.find_nodes_by_kind("pair");
        for (i, pair) in pairs.iter().take(3).enumerate() {
            println!("   Pair {}: {}", i + 1, pair.text().chars().take(50).collect::<String>());
            for child in &pair.children {
                if let Some(field_name) = child.field_name() {
                    let text_preview = child.text().chars().take(30).collect::<String>();
                    println!("     - {}: {}", field_name, text_preview);
                }
            }
        }
    }
    
    // 6. 错误检测
    println!("\n6. 错误检测:");
    fn find_errors(node: &TreeCursorSyntaxNode, path: &str, count: &mut usize) {
        if *count >= 5 { return; } // 限制输出数量
        if node.has_error() {
            println!("   错误节点: {} at {}", node.kind, path);
            *count += 1;
        }
        for (i, child) in node.children.iter().enumerate() {
            find_errors(child, &format!("{}.{}", path, i), count);
        }
    }
    let mut error_count = 0;
    find_errors(&cst, "root", &mut error_count);
    if error_count == 0 {
        println!("   ✓ 未发现语法错误");
    }
    
    // 7. 演示共享源码优化
    println!("\n7. 内存优化:");
    println!("   源码共享: 所有节点共享同一份源码，减少内存占用");
    println!("   Arc 引用计数: {}", std::sync::Arc::strong_count(cst.shared_source()));
    
    // 8. 性能统计
    println!("\n8. 性能统计:");
    let total_nodes = cst.preorder().count();
    println!("   总节点数: {}", total_nodes);
    println!("   平均节点大小: {:.1} 字节", source.len() as f64 / total_nodes as f64);
}

/// 向后兼容的演示函数（默认 JSON）
/// 
/// # Arguments
/// * `json_source` - 要演示的 JSON 字符串
pub fn demonstrate_cst_features(json_source: &str) {
    demonstrate_cst_features_multi_format(json_source, Some(SourceType::Json));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    /// 测试新的构建器 API
    #[test]
    fn test_builder_api() {
        let src = r#"{ "foo": [1, 2, 3] }"#;
        
        // 测试基本解析
        let cst = CstParser::parse(src);
        assert!(!cst.children.is_empty());
        
        // 测试构建器风格的遍历
        let preorder_count = cst.preorder().count();
        let postorder_count = cst.postorder().count();
        let breadth_first_count = cst.breadth_first().count();
        
        // 所有遍历方式应该访问相同数量的节点
        assert_eq!(preorder_count, postorder_count);
        assert_eq!(postorder_count, breadth_first_count);
        
        // 测试链式操作
        let strings: Vec<_> = cst.find_nodes_by_kind("string")
            .into_iter()
            .map(|node| node.text())
            .collect();
        assert!(!strings.is_empty());
    }

    /// 一个非常简单的 JSON，测试最顶层一定能解析成一个有孩子的节点，
    /// 并且第一个孩子是 `object` 类型。
    #[test]
    fn test_parse_object() {
        let src = r#"{ "foo": 42 }"#;
        let cst = CstParser::parse(src);

        // 顶层一定有孩子
        assert!(!cst.children.is_empty(), "root should have children");

        // 第一个孩子就是 JSON object
        let obj = &cst.children[0];
        assert_eq!(obj.kind, "object");
        // 且它的 text 应该恰好是原文去掉最外面空白后对应那段
        assert_eq!(obj.text().trim(), r#"{ "foo": 42 }"#);

        // object 下应该有至少一个 pair
        assert!(obj.children.iter().any(|n| n.kind == "pair"), "object should contain a pair");
    }

    /// 测试数组
    #[test]
    fn test_parse_array() {
        let src = r#"[1, true, null]"#;
        let cst = CstParser::parse(src);
        let arr = &cst.children[0];
        assert_eq!(arr.kind, "array");

        // 数组有 3 个元素
        // tree-sitter-json CST 中，literal 节点通常直接就是子节点
        let literal_kinds: Vec<_> = arr
            .children
            .iter()
            // 过滤出数字、true、null
            .filter(|n| ["number", "true", "null"].contains(&n.kind.as_str()))
            .map(|n| &n.kind)
            .collect();
        assert_eq!(literal_kinds, &["number", "true", "null"]);
    }

    /// 嵌套测试
    #[test]
    fn test_nested() {
        let src = r#"{ "a": [ { "b": "c" } ] }"#;
        let cst = CstParser::parse(src);

        // 路径 cst.children[0] → "object"
        let obj = &cst.children[0];
        // 它下面会有一个 pair
        let pair = obj
            .children
            .iter()
            .find(|n| n.kind == "pair")
            .expect("object must contain a pair");
        // pair 的 value 应该是 array
        // 由于 `from_cursor` 只记录它的 text 和 kind，真正的 value node 在 children 里
        let array_node = pair
            .children
            .iter()
            .find(|n| n.kind == "array")
            .expect("pair should contain an array");
        assert!(!array_node.children.is_empty());

        // 最深处应该有一个 inner object
        let inner_obj = array_node
            .children
            .iter()
            .find(|n| n.kind == "object")
            .expect("array should contain an object");
        assert!(inner_obj
            .children
            .iter()
            .any(|n| n.kind == "pair" && n.text().contains("\"b\"")));
    }

    /// 测试字段名记录
    #[test]
    fn test_field_names() {
        let src = r#"{ "key": "value" }"#;
        let cst = CstParser::parse(src);
        let obj = &cst.children[0];
        
        // 找到 pair 节点
        let pair = obj.children.iter().find(|n| n.kind == "pair").unwrap();
        
        // pair 的子节点应该有 key 和 value 字段
        let key_node = pair.children.iter().find(|n| n.field_name() == Some("key"));
        let value_node = pair.children.iter().find(|n| n.field_name() == Some("value"));
        
        assert!(key_node.is_some(), "Should have key field");
        assert!(value_node.is_some(), "Should have value field");
        
        if let Some(key) = key_node {
            assert_eq!(key.text().as_ref(), r#""key""#);
        }
        if let Some(value) = value_node {
            assert_eq!(value.text().as_ref(), r#""value""#);
        }
    }

    /// 测试错误节点处理
    #[test]
    fn test_error_handling() {
        let src = r#"{ "incomplete": }"#; // 故意的语法错误
        let cst = CstParser::parse(src);
        
        // 应该能检测到错误
        fn has_error_in_tree(node: &TreeCursorSyntaxNode) -> bool {
            if node.has_error() {
                return true;
            }
            node.children.iter().any(has_error_in_tree)
        }
        
        assert!(has_error_in_tree(&cst), "Should detect syntax error in malformed JSON");
    }

    /// 测试迭代器遍历
    #[test]
    fn test_iterators() {
        let src = r#"{ "a": [1, 2] }"#;
        let cst = CstParser::parse(src);
        
        // 测试前序遍历
        let preorder_kinds: Vec<String> = cst
            .preorder()
            .map(|node| node.kind.clone())
            .collect();
        
        // 前序遍历应该先访问父节点再访问子节点
        assert!(preorder_kinds.contains(&"document".to_string()));
        assert!(preorder_kinds.contains(&"object".to_string()));
        assert!(preorder_kinds.contains(&"array".to_string()));
        
        // 测试后序遍历
        let postorder_kinds: Vec<String> = cst
            .postorder()
            .map(|node| node.kind.clone())
            .collect();
        
        // 后序遍历应该先访问子节点再访问父节点
        assert!(postorder_kinds.contains(&"document".to_string()));
        assert!(postorder_kinds.contains(&"object".to_string()));
        
        // 测试广度优先遍历
        let breadth_first_kinds: Vec<String> = cst
            .breadth_first()
            .map(|node| node.kind.clone())
            .collect();
        
        assert!(breadth_first_kinds.contains(&"document".to_string()));
        assert!(breadth_first_kinds.contains(&"object".to_string()));
        
        // 测试查找特定节点
        let numbers = cst.find_nodes_by_kind("number");
        assert_eq!(numbers.len(), 2); // 应该找到 1 和 2
    }

    /// 测试注释和空白处理
    #[test]
    fn test_whitespace_and_comments() {
        // JSON 标准不支持注释，但我们测试空白字符的处理
        let src = r#"
        {
            "key"  :   "value"  
        }
        "#;
        let cst = CstParser::parse(src);
        let obj = &cst.children[0];
        assert_eq!(obj.kind, "object");
        
        // 应该能正确解析即使有额外空白
        let pair = obj.children.iter().find(|n| n.kind == "pair").unwrap();
        assert!(pair.children.iter().any(|n| n.text().trim() == r#""key""#));
    }

    /// 测试深度嵌套性能
    #[test]
    fn test_deep_nesting() {
        // 创建深度嵌套的 JSON
        let mut src = String::new();
        let depth = 100;
        
        // 构建深度嵌套的数组
        for _ in 0..depth {
            src.push('[');
        }
        src.push_str("42");
        for _ in 0..depth {
            src.push(']');
        }
        
        let start = std::time::Instant::now();
        let cst = CstParser::parse(&src);
        let duration = start.elapsed();
        
        // 验证解析成功且性能合理（应该在几毫秒内完成）
        assert!(!cst.children.is_empty());
        assert!(duration.as_millis() < 100, "Deep nesting should parse quickly");
        
        // 验证嵌套深度
        fn count_depth(node: &TreeCursorSyntaxNode) -> usize {
            if node.children.is_empty() {
                1
            } else {
                1 + node.children.iter().map(count_depth).max().unwrap_or(0)
            }
        }
        
        let actual_depth = count_depth(&cst);
        assert!(actual_depth > 50, "Should handle deep nesting");
    }

    /// 测试不完整 JSON 的错误处理
    #[test]
    fn test_incomplete_json_cases() {
        let test_cases = vec![
            r#"{ "key": }"#,           // 缺少值
            r#"{ "key" "value" }"#,    // 缺少冒号
            r#"{ "key": "value" "#,    // 缺少结束大括号
            r#"[1, 2, ]"#,            // trailing comma
            r#""unterminated string"#, // 未终止的字符串
        ];
        
        for (i, src) in test_cases.iter().enumerate() {
            let cst = CstParser::parse(src);
            
            // 检查是否检测到错误
            fn has_error_recursive(node: &TreeCursorSyntaxNode) -> bool {
                node.has_error() || node.children.iter().any(has_error_recursive)
            }
            
            assert!(
                has_error_recursive(&cst),
                "Test case {} should detect error in: {}",
                i, src
            );
        }
    }

    /// 测试特殊字符和 Unicode
    #[test]
    fn test_special_characters() {
        let src = r#"{ "emoji": "🚀", "chinese": "你好", "escape": "line1\nline2" }"#;
        let cst = CstParser::parse(src);
        let obj = &cst.children[0];
        
        // 应该能正确处理 Unicode 字符
        let pairs = obj.find_nodes_by_kind("pair");
        assert_eq!(pairs.len(), 3);
        
        // 验证能正确提取包含特殊字符的文本
        let emoji_pair = pairs.iter()
            .find(|p| p.text().contains("emoji"))
            .expect("Should find emoji pair");
        assert!(emoji_pair.text().contains("🚀"));
    }

    /// 测试边界情况
    #[test]
    fn test_edge_cases() {
        let test_cases = vec![
            ("", "空字符串"),
            ("{}", "空对象"),
            ("[]", "空数组"),
            ("null", "null 值"),
            ("true", "boolean true"),
            ("false", "boolean false"),
            ("0", "数字 0"),
            (r#""""#, "空字符串"),
            (r#"{"":""}"#, "空键空值"),
        ];
        
        for (src, description) in test_cases {
            let cst = CstParser::parse(src);
            
            // 所有这些都应该是有效的 JSON，不应该有错误
            fn has_error_recursive(node: &TreeCursorSyntaxNode) -> bool {
                node.has_error() || node.children.iter().any(has_error_recursive)
            }
            
            if !src.is_empty() {
                assert!(
                    !has_error_recursive(&cst),
                    "{} should parse without errors",
                    description
                );
            }
        }
    }

    /// 测试演示功能
    #[test]
    fn test_demonstration() {
        let json = r#"{ "name": "CST Demo", "version": 1.0, "features": ["parsing", "iteration"] }"#;
        
        // 这个测试主要确保演示函数不会 panic
        // 在实际使用中，这个函数会打印到 stdout
        demonstrate_cst_features(json);
        
        // 验证基本功能仍然工作
        let cst = CstParser::parse(json);
        assert!(!cst.has_error());
        assert!(!cst.children.is_empty());
    }

    /// 测试超长字符串和极端 Unicode
    #[test]
    fn test_extreme_cases() {
        // 测试超长字符串
        let long_string = "a".repeat(10000);
        let long_json = format!(r#"{{"key": "{}"}}"#, long_string);
        let cst = CstParser::parse(&long_json);
        assert!(!cst.has_error());
        
        // 验证能正确处理长字符串
        let strings = cst.find_nodes_by_kind("string_content");
        let long_content = strings.iter().find(|s| s.text().len() > 5000);
        assert!(long_content.is_some(), "Should handle very long strings");
        
        // 测试极端 Unicode 转义
        let unicode_json = r#"{"emoji": "\ud83d\ude80", "chinese": "\u4f60\u597d", "complex": "\ud83c\udf08\ud83e\udd84"}"#;
        let cst = CstParser::parse(unicode_json);
        assert!(!cst.has_error());
        
        // 测试各种转义字符
        let escape_json = r#"{"escapes": "\"\\\/\b\f\n\r\t"}"#;
        let cst = CstParser::parse(escape_json);
        assert!(!cst.has_error());
    }

    /// 测试极深嵌套（压力测试）
    #[test]
    fn test_extreme_nesting() {
        // 测试超深数组嵌套
        let depth = 1000;
        let mut json = String::new();
        for _ in 0..depth {
            json.push('[');
        }
        json.push_str("null");
        for _ in 0..depth {
            json.push(']');
        }
        
        let start = std::time::Instant::now();
        let cst = CstParser::parse(&json);
        let duration = start.elapsed();
        
        assert!(!cst.has_error());
        assert!(duration.as_millis() < 1000, "Should handle extreme nesting efficiently");
        
        // 测试极深对象嵌套
        let mut obj_json = String::new();
        for i in 0..500 {
            obj_json.push_str(&format!(r#"{{"level{}":"#, i));
        }
        obj_json.push_str("\"value\"");
        for _ in 0..500 {
            obj_json.push('}');
        }
        
        let cst = CstParser::parse(&obj_json);
        assert!(!cst.has_error());
    }

    /// 测试并发安全性
    #[test]
    fn test_concurrent_parsing() {
        use std::thread;
        use std::sync::Arc;
        
        let test_cases = vec![
            r#"{"thread": 1, "data": [1, 2, 3]}"#,
            r#"{"thread": 2, "data": {"nested": true}}"#,
            r#"{"thread": 3, "data": "string value"}"#,
            r#"{"thread": 4, "data": null}"#,
        ];
        
        let test_cases = Arc::new(test_cases);
        let mut handles = vec![];
        
        // 启动多个线程同时解析
        for i in 0..4 {
            let cases = test_cases.clone();
            let handle = thread::spawn(move || {
                let json = &cases[i];
                let cst = CstParser::parse(json);
                assert!(!cst.has_error());
                
                // 验证每个线程都有自己的 parser
                let objects = cst.find_nodes_by_kind("object");
                assert!(!objects.is_empty());
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// 测试内存效率（Arc 共享）
    #[test]
    fn test_memory_efficiency() {
        let json = r#"{"large": "data", "with": ["many", "nested", {"objects": true}]}"#;
        let cst = CstParser::parse(json);
        
        // 验证所有节点共享同一份源码
        let initial_count = std::sync::Arc::strong_count(cst.shared_source());
        
        // 遍历所有节点，它们应该都共享同一个 Arc
        let mut node_count = 0;
        for _node in cst.preorder() {
            node_count += 1;
        }
        
        // Arc 引用计数应该等于节点数量 + 1（根节点）
        assert!(node_count > 10, "Should have multiple nodes");
        assert!(initial_count > 1, "Source should be shared among nodes");
        
        println!("节点数量: {}, Arc 引用计数: {}", node_count, initial_count);
    }

    /// 测试错误恢复能力
    #[test]
    fn test_error_recovery() {
        let malformed_cases = vec![
            (r#"{"key": value}"#, "未引用的值"),
            (r#"{"key": "value",}"#, "尾随逗号"),
            (r#"{key: "value"}"#, "未引用的键"),
            (r#"{"key": "value" "another": "value"}"#, "缺少逗号"),
            (r#"[1, 2, 3,]"#, "数组尾随逗号"),
            (r#"{"nested": {"incomplete": }"#, "不完整的嵌套"),
        ];
        
        for (json, description) in malformed_cases {
            let cst = CstParser::parse(json);
            
            // 应该能检测到错误但不崩溃
            fn has_any_error(node: &TreeCursorSyntaxNode) -> bool {
                if node.has_error() {
                    return true;
                }
                node.children.iter().any(has_any_error)
            }
            
            assert!(has_any_error(&cst), "应该检测到错误: {}", description);
            
            // 即使有错误，也应该能遍历树结构
            let node_count = cst.preorder().count();
            assert!(node_count > 0, "即使有错误也应该能构建部分树结构");
        }
    }

    /// 测试 SourceType 枚举功能
    #[test]
    fn test_source_type() {
        // 测试从扩展名推断类型
        assert_eq!(SourceType::from_extension("json"), Some(SourceType::Json));
        assert_eq!(SourceType::from_extension("JSON"), Some(SourceType::Json));
        assert_eq!(SourceType::from_extension("yaml"), Some(SourceType::Yaml));
        assert_eq!(SourceType::from_extension("yml"), Some(SourceType::Yaml));
        assert_eq!(SourceType::from_extension("YAML"), Some(SourceType::Yaml));
        assert_eq!(SourceType::from_extension("txt"), None);
        assert_eq!(SourceType::from_extension(""), None);
        
        // 测试显示名称
        assert_eq!(SourceType::Json.display_name(), "JSON");
        assert_eq!(SourceType::Yaml.display_name(), "YAML");
        
        // 测试相等性
        assert_eq!(SourceType::Json, SourceType::Json);
        assert_eq!(SourceType::Yaml, SourceType::Yaml);
        assert_ne!(SourceType::Json, SourceType::Yaml);
    }

    /// 测试 JSON 解析功能
    #[test]
    fn test_json_parsing() {
        let json_src = r#"{"name": "test", "values": [1, 2, 3], "nested": {"key": "value"}}"#;
        
        // 使用默认方法（应该是 JSON）
        let cst1 = CstParser::parse(json_src);
        assert!(!cst1.has_error());
        
        // 明确指定 JSON
        let cst2 = CstParser::parse_as(json_src, SourceType::Json);
        assert!(!cst2.has_error());
        
        // 验证结构
        assert!(!cst2.children.is_empty());
        let obj = &cst2.children[0];
        assert_eq!(obj.kind, "object");
        
        // 查找特定节点
        let strings = cst2.find_nodes_by_kind("string");
        assert!(!strings.is_empty());
        
        let numbers = cst2.find_nodes_by_kind("number");
        assert_eq!(numbers.len(), 3); // 1, 2, 3
    }

    /// 测试 YAML 解析功能
    #[test]
    fn test_yaml_parsing() {
        let yaml_src = r#"
name: test
values:
  - 1
  - 2
  - 3
nested:
  key: value
"#;
        
        // 明确指定 YAML
        let cst = CstParser::parse_as(yaml_src, SourceType::Yaml);
        assert!(!cst.has_error());
        
        // 验证根节点类型
        assert_eq!(cst.kind, "stream");
        
        // YAML 应该有 document 子节点
        let documents = cst.find_nodes_by_kind("document");
        assert!(!documents.is_empty());
        
        // 查找 YAML 特有的节点类型
        let block_mappings = cst.find_nodes_by_kind("block_mapping");
        assert!(!block_mappings.is_empty());
        
        let block_sequences = cst.find_nodes_by_kind("block_sequence");
        assert!(!block_sequences.is_empty());
    }

    /// 测试智能解析功能
    #[test]
    fn test_smart_parsing() {
        // 测试 JSON 检测
        let json_src = r#"{"key": "value"}"#;
        let (cst, detected_type) = CstParser::parse_smart(json_src);
        assert_eq!(detected_type, SourceType::Json);
        assert!(!cst.has_error());
        
        // 测试 YAML 检测（当 JSON 解析失败时）
        let yaml_src = "key: value\nlist:\n  - item1\n  - item2";
        let (cst, detected_type) = CstParser::parse_smart(yaml_src);
        assert_eq!(detected_type, SourceType::Yaml);
        assert!(!cst.has_error());
        
        // 测试明显的 JSON（带花括号）
        let json_array = r#"[1, 2, 3]"#;
        let (cst, detected_type) = CstParser::parse_smart(json_array);
        assert_eq!(detected_type, SourceType::Json);
        assert!(!cst.has_error());
    }

    /// 测试并发多格式解析
    #[test]
    fn test_concurrent_multi_format_parsing() {
        use std::thread;
        use std::sync::Arc;
        
        let test_cases = vec![
            (r#"{"json": "data"}"#, SourceType::Json),
            ("yaml: data", SourceType::Yaml),
            (r#"[1, 2, 3]"#, SourceType::Json),
            ("list:\n  - item1\n  - item2", SourceType::Yaml),
        ];
        
        let test_cases = Arc::new(test_cases);
        let mut handles = vec![];
        
        // 启动多个线程同时解析不同格式
        for i in 0..4 {
            let cases = test_cases.clone();
            let handle = thread::spawn(move || {
                let (source, source_type) = &cases[i];
                let cst = CstParser::parse_as(source, *source_type);
                assert!(!cst.has_error());
                
                // 验证解析结果
                assert!(!cst.children.is_empty());
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
    }

    /// 测试 YAML 特有功能
    #[test]
    fn test_yaml_specific_features() {
        // 测试多文档 YAML
        let multi_doc_yaml = r#"
---
doc1: value1
---
doc2: value2
"#;
        
        let cst = CstParser::parse_as(multi_doc_yaml, SourceType::Yaml);
        assert!(!cst.has_error());
        
        // 应该有多个 document 节点
        let documents = cst.find_nodes_by_kind("document");
        assert!(documents.len() >= 2);
        
        // 测试 YAML 列表语法
        let yaml_list = r#"
items:
  - name: item1
    value: 100
  - name: item2
    value: 200
"#;
        
        let cst = CstParser::parse_as(yaml_list, SourceType::Yaml);
        assert!(!cst.has_error());
        
        let block_sequences = cst.find_nodes_by_kind("block_sequence");
        assert!(!block_sequences.is_empty());
        
        let block_mappings = cst.find_nodes_by_kind("block_mapping");
        assert!(!block_mappings.is_empty());
    }

    /// 测试错误格式处理
    #[test]
    fn test_format_error_handling() {
        // 测试无效的 JSON
        let invalid_json = r#"{"key": value}"#; // 未引用的值
        let cst = CstParser::parse_as(invalid_json, SourceType::Json);
        
        // 应该检测到错误
        fn has_error_recursive(node: &TreeCursorSyntaxNode) -> bool {
            node.has_error() || node.children.iter().any(has_error_recursive)
        }
        assert!(has_error_recursive(&cst));
        
        // 测试无效的 YAML
        let invalid_yaml = "key: value\n  invalid indentation";
        let cst = CstParser::parse_as(invalid_yaml, SourceType::Yaml);
        
        // YAML 解析器通常更宽容，但仍应能处理
        // 即使有错误也应该能构建部分树
        assert!(!cst.children.is_empty());
    }

    /// 测试构建器 API 与多格式支持
    #[test]
    fn test_builder_api_multi_format() {
        // JSON 构建器风格
        let json_src = r#"{"items": ["a", "b", "c"]}"#;
        let json_cst = CstParser::parse_as(json_src, SourceType::Json);
        
        let json_strings: Vec<_> = json_cst
            .preorder()
            .filter(|node| node.kind == "string")
            .map(|node| node.text())
            .collect();
        assert!(!json_strings.is_empty());
        
        // YAML 构建器风格
        let yaml_src = r#"
items:
  - a
  - b
  - c
"#;
        let yaml_cst = CstParser::parse_as(yaml_src, SourceType::Yaml);
        
        let yaml_scalars: Vec<_> = yaml_cst
            .breadth_first()
            .filter(|node| node.kind == "plain_scalar")
            .map(|node| node.text())
            .collect();
        assert!(!yaml_scalars.is_empty());
    }

    /// 测试 YAML 高级特性和边界情况
    #[test]
    fn test_yaml_advanced_features() {
        // 测试多文档 YAML 与注释
        let multi_doc_with_comments = r#"
# 第一个文档
---
name: "Document 1"
items:
  - item1  # 行内注释
  - item2
  # 这是注释行
metadata:
  version: 1.0

# 第二个文档
---
name: "Document 2"
items:
  - item3
  - item4
metadata:
  version: 2.0
"#;
        
        let cst = CstParser::parse_as(multi_doc_with_comments, SourceType::Yaml);
        assert!(!cst.has_error());
        
        // 应该有多个 document 节点
        let documents = cst.find_nodes_by_kind("document");
        assert!(documents.len() >= 2, "应该有至少 2 个文档");
        
        // 测试注释节点
        let comments = cst.find_nodes_by_kind("comment");
        assert!(!comments.is_empty(), "应该找到注释节点");
        
        // 测试复杂的缩进和嵌套
        let complex_yaml = r#"
root:
  level1:
    level2:
      - array_item1:
          nested_key: value1
      - array_item2:
          nested_key: value2
    another_level2:
      key: value
  another_level1:
    - simple_item
    - complex_item:
        sub_key: sub_value
"#;
        
        let cst = CstParser::parse_as(complex_yaml, SourceType::Yaml);
        assert!(!cst.has_error());
        
        let block_mappings = cst.find_nodes_by_kind("block_mapping");
        assert!(!block_mappings.is_empty(), "应该有块映射");
        
        let block_sequences = cst.find_nodes_by_kind("block_sequence");
        assert!(!block_sequences.is_empty(), "应该有块序列");
    }
    
    /// 测试 YAML 中的各种标量类型
    #[test]
    fn test_yaml_scalar_types() {
        let yaml_scalars = r#"
string_plain: plain string
string_quoted: "quoted string"
string_single: 'single quoted'
number_int: 42
number_float: 3.14
boolean_true: true
boolean_false: false
null_value: null
empty_value: 
multiline: |
  This is a multiline
  string using literal
  block scalar style
folded: >
  This is a folded
  string that will be
  joined on a single line
"#;
        
        let cst = CstParser::parse_as(yaml_scalars, SourceType::Yaml);
        assert!(!cst.has_error());
        
        // 测试不同类型的标量（基于实际的节点类型）
        let plain_scalars = cst.find_nodes_by_kind("plain_scalar");
        assert!(!plain_scalars.is_empty(), "应该有普通标量");
        
        let double_quoted_scalars = cst.find_nodes_by_kind("double_quote_scalar");
        assert!(!double_quoted_scalars.is_empty(), "应该有双引号标量");
        
        let single_quoted_scalars = cst.find_nodes_by_kind("single_quote_scalar");
        assert!(!single_quoted_scalars.is_empty(), "应该有单引号标量");
        
        let block_scalars = cst.find_nodes_by_kind("block_scalar");
        assert!(!block_scalars.is_empty(), "应该有块标量");
        
        let integer_scalars = cst.find_nodes_by_kind("integer_scalar");
        assert!(!integer_scalars.is_empty(), "应该有整数标量");
        
        let float_scalars = cst.find_nodes_by_kind("float_scalar");
        assert!(!float_scalars.is_empty(), "应该有浮点数标量");
        
        let boolean_scalars = cst.find_nodes_by_kind("boolean_scalar");
        assert!(!boolean_scalars.is_empty(), "应该有布尔标量");
        
        let null_scalars = cst.find_nodes_by_kind("null_scalar");
        assert!(!null_scalars.is_empty(), "应该有空值标量");
    }
    
    /// 测试 YAML 错误恢复和边界情况
    #[test]
    fn test_yaml_error_cases() {
        let problematic_cases = vec![
            // 缩进不一致
            ("inconsistent_indent", r#"
items:
  - item1
    - item2  # 错误的缩进
"#),
            // 混合制表符和空格
            ("mixed_tabs_spaces", "items:\n\t- item1\n  - item2"),
            // 未终止的引号
            ("unterminated_quote", r#"key: "unterminated string"#),
            // 无效的键值对
            ("invalid_mapping", "key1: value1\nkey2 value2"),  // 缺少冒号
        ];
        
        for (description, yaml) in problematic_cases {
            let cst = CstParser::parse_as(yaml, SourceType::Yaml);
            
            // YAML 解析器通常比 JSON 更宽容，但仍应能处理
            // 即使有错误也应该能构建部分树
            assert!(!cst.children.is_empty(), "即使有错误也应该构建部分树: {}", description);
            
            // 可以检查是否有错误节点
            fn has_any_error(node: &TreeCursorSyntaxNode) -> bool {
                if node.has_error() {
                    return true;
                }
                node.children.iter().any(has_any_error)
            }
            
            // 某些情况下可能检测到错误
            let has_error = has_any_error(&cst);
            println!("YAML 案例 '{}' 错误检测: {}", description, has_error);
        }
    }
    
    /// 测试智能检测的准确性
    #[test]
    fn test_smart_detection_accuracy() {
        let test_cases = vec![
            // 明显的 JSON 案例
            (r#"{"json": true}"#, SourceType::Json, "JSON 对象"),
            (r#"[1, 2, 3]"#, SourceType::Json, "JSON 数组"),
            (r#"{"nested": {"deep": "value"}}"#, SourceType::Json, "嵌套 JSON"),
            
            // 明显的 YAML 案例  
            ("key: value", SourceType::Yaml, "简单 YAML 映射"),
            ("---\nkey: value", SourceType::Yaml, "YAML 文档分隔符"),
            ("- item1\n- item2", SourceType::Yaml, "YAML 列表"),
            ("key:\n  nested: value", SourceType::Yaml, "嵌套 YAML"),
            
            // 边界案例
            ("key:value", SourceType::Yaml, "无空格的 YAML"),
            ("# 纯注释\nkey: value", SourceType::Yaml, "带注释的 YAML"),
            (r#"{"key":123}"#, SourceType::Json, "紧凑 JSON"),
        ];
        
        for (source, expected_type, description) in test_cases {
            let detected_type = SourceType::detect_from_content(source);
            assert_eq!(
                detected_type, expected_type,
                "检测失败: {} - 期望 {}, 实际 {}",
                description, expected_type.display_name(), detected_type.display_name()
            );
            
            // 验证智能解析也能正确处理
            let (cst, smart_detected) = CstParser::parse_smart(source);
            assert!(!cst.has_error(), "智能解析失败: {}", description);
            
            // 智能解析的结果应该与启发式检测一致，或者是备用方案
            if smart_detected != expected_type {
                println!("智能解析使用备用方案: {} -> {}", 
                        expected_type.display_name(), smart_detected.display_name());
            }
        }
    }
    
    /// 测试内存优化效果
    #[test]
    fn test_memory_optimization_arc_str() {
        let large_json = format!(r#"{{
  "description": "测试 Arc<str> 内存优化",
  "data": [{}],
  "metadata": {{
    "count": {},
    "memory_optimized": true
  }}
}}"#, 
            (0..50).map(|i| format!(r#"{{"id": {}, "name": "item{}", "value": "{}"}}"#, i, i, "x".repeat(10)))
                    .collect::<Vec<_>>().join(", "),
            50
        );
        
        let cst = CstParser::parse(&large_json);
        
        // 验证所有节点共享同一个 Arc<str>
        let source_arc = cst.shared_source();
        let initial_count = Arc::strong_count(source_arc);
        
        // 遍历所有节点，验证它们都共享相同的源码
        let mut node_count = 0;
        let mut shared_count = 0;
        
        for node in cst.iter_preorder() {
            node_count += 1;
            if Arc::ptr_eq(node.shared_source(), source_arc) {
                shared_count += 1;
            }
        }
        
        assert_eq!(node_count, shared_count, "所有节点都应该共享相同的源码");
        assert!(initial_count > 10, "Arc 引用计数应该反映节点数量");
        
        // 验证 Arc<str> 的零拷贝文本提取
        let strings = cst.find_nodes_by_kind("string");
        for string_node in strings.iter().take(5) {
            let text = string_node.text();
            // text() 返回的应该是 Cow::Borrowed，表示零拷贝
            match text {
                Cow::Borrowed(_) => {
                    // 这是我们想要的：零拷贝
                }
                Cow::Owned(_) => {
                    panic!("意外的拷贝发生：{}", text);
                }
            }
        }
        
        println!("内存优化验证: {} 个节点共享同一份源码", node_count);
        println!("Arc<str> 引用计数: {}", initial_count);
    }
}