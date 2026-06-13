use std::collections::HashMap;

/// 对话轮次记录
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub user_text: String,
    pub assistant_text: String,
    /// 这一轮操作后新创建的节点 ID 列表
    pub created_nodes: Vec<String>,
}

/// 对话上下文管理器
pub struct ConversationContext {
    pub turns: Vec<ConversationTurn>,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self { turns: Vec::new() }
    }

    /// 添加一轮对话
    pub fn add_turn(&mut self, user: String, assistant: String, nodes: Vec<String>) {
        self.turns.push(ConversationTurn {
            user_text: user,
            assistant_text: assistant,
            created_nodes: nodes,
        });
    }

    /// 获取最近 N 轮对话的摘要
    pub fn get_recent_summary(&self, n: usize) -> String {
        if self.turns.is_empty() {
            return "（无历史对话）".into();
        }
        let start = if self.turns.len() > n {
            self.turns.len() - n
        } else {
            0
        };
        self.turns[start..]
            .iter()
            .enumerate()
            .map(|(i, turn)| {
                format!(
                    "轮{}: 用户: {} | 助手: {}",
                    i + 1,
                    turn.user_text,
                    turn.assistant_text
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 将对话历史导出为 (role, content) 列表，供 LLM Scheduler 使用
    pub fn to_history(&self) -> Vec<(String, String)> {
        self.turns
            .iter()
            .flat_map(|turn| {
                vec![
                    ("user".to_string(), turn.user_text.clone()),
                    ("assistant".to_string(), turn.assistant_text.clone()),
                ]
            })
            .collect()
    }

    /// 获取最近一次用户输入的文本
    pub fn last_user_text(&self) -> Option<String> {
        self.turns.last().map(|t| t.user_text.clone())
    }

    /// 在历史中搜索可能的节点引用
    /// 比如用户说「那个菱形」，找到上一轮中创建的 Decision 类型节点
    pub fn resolve_reference(
        &self,
        text: &str,
        nodes: &HashMap<String, crate::engine::canvas_state::DiagramNode>,
    ) -> Option<(String, String)> {
        // 查找模糊引用词
        let ref_words = ["那个", "这个", "它", "上面那个", "刚才那个"];
        let has_ref = ref_words.iter().any(|w| text.contains(w));
        if !has_ref {
            return None;
        }

        // 在上一轮的节点中搜索
        if let Some(last_turn) = self.turns.last() {
            for node_id in &last_turn.created_nodes {
                if let Some(node) = nodes.get(node_id) {
                    // 匹配提到的类型关键词
                    let type_keywords = [
                        ("菱形", "Decision"),
                        ("矩形", "Process"),
                        ("圆角", "Start"),
                        ("圆形", "Start"),
                        ("开始", "Start"),
                        ("结束", "End"),
                        ("判断", "Decision"),
                        ("处理", "Process"),
                    ];
                    for (keyword, nt) in &type_keywords {
                        if text.contains(keyword)
                            && format!("{:?}", node.node_type) == *nt
                        {
                            return Some((node_id.clone(), node.label.clone()));
                        }
                    }
                    // 如果只有代词没有类型，返回上一轮最后一个节点
                    return Some((node_id.clone(), node.label.clone()));
                }
            }
        }
        None
    }

    /// 丰富用户文本：消解代词引用，附加上下文信息
    /// 例如「把那个菱形改成绿色」→ 「把那个菱形改成绿色
    /// （注：'那个菱形'指的是节点'判断条件'(id: abc123)）」
    pub fn enrich_user_text(
        &self,
        text: &str,
        nodes: &HashMap<String, crate::engine::canvas_state::DiagramNode>,
    ) -> String {
        match self.resolve_reference(text, nodes) {
            Some((node_id, label)) => {
                format!(
                    "{}\n（上下文：上一轮创建的节点 \"{}\" (id: {})）",
                    text, label, node_id
                )
            }
            None => text.to_string(),
        }
    }
}
