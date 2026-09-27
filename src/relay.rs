//! 二段階踏み台(Hop1/Hop2)の「利用アプリ許可リスト」と
//! 「ノードの定期ランダムローテーション」の最小実装。
//!
//! 設計方針(`README.md`「踏み台の用途スコープ」「Hop1/Hop2ノードの定期的な
//! ランダムローテーション」参照):
//! - 踏み台はaon-co-jp運営の自社アプリの通信のみを中継し、任意の一般
//!   インターネット宛先への中継は拒否する(許可リスト方式)。
//! - Hop1/Hop2のノードは固定せず、プールから定期的にランダムに入れ替える
//!   (特定国のファイアウォール/DPIによるブロックリスト化への対策)。

use std::collections::HashSet;

/// 踏み台の利用を許可されたアプリのID。
///
/// 2026-09-27時点でaon-co-jpが運営する主要アプリのみを列挙する。新しい
/// 自社アプリを追加する場合はここに追記する(このリストに無いapp_idは
/// 常に拒否される)。
const ALLOWED_APP_IDS: &[&str] = &["open-tv-chat", "open-english", "aruaru-tokyo"];

/// 指定した`app_id`がこの踏み台の利用を許可されているかを判定する。
///
/// 一般インターネットの任意サイト(許可リストに無いapp_id)への中継は
/// 拒否する。これにより、踏み台の法的性質を「自社サービス群共通の通信経路」
/// に保ち、任意サイトを代理する汎用VPNにはしない。
pub fn is_app_allowed(app_id: &str) -> bool {
    ALLOWED_APP_IDS.contains(&app_id)
}

/// Hop1またはHop2として稼働する中継ノードの識別子(エンドポイント)。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelayNode {
    pub endpoint: String,
}

impl RelayNode {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

/// Hop1用またはHop2用のノードプール。
///
/// ノードを固定せず、プールから乱数でノードを選ぶことで、特定ノードへの
/// アクセス集中や、外部からの「既知の中継ノード」特定を難しくする。
/// `rotate`でプールの内容を定期的に入れ替える(古いノードの退役・新規
/// ノードの追加)。実際の乱数間隔・ノード数はAI判断で実装時に詳細化する
/// (`PORTING.md`参照)。
#[derive(Debug, Clone, Default)]
pub struct NodePool {
    nodes: Vec<RelayNode>,
}

impl NodePool {
    pub fn new(nodes: Vec<RelayNode>) -> Self {
        Self { nodes }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 呼び出し側が与えた乱数(`0.0..1.0`)を使ってプールから1ノードを選ぶ。
    ///
    /// 乱数生成そのものはこの最小実装のスコープ外とし(実装着手時に
    /// 暗号論的に安全な乱数源を選定する)、呼び出し側から`[0.0, 1.0)`の
    /// 値を渡してもらう形にすることで、この関数自体は決定的にテストできる
    /// ようにしている。
    pub fn pick(&self, random_unit: f64) -> Option<&RelayNode> {
        if self.nodes.is_empty() {
            return None;
        }
        let index = ((random_unit.clamp(0.0, 0.999_999_999)) * self.nodes.len() as f64) as usize;
        self.nodes.get(index)
    }

    /// プールを新しいノード集合に入れ替える(ローテーション)。
    ///
    /// 通話中のセッションはこの入れ替えの影響を受けない(既存セッションは
    /// 呼び出し時に選ばれたノードを使い続ける想定、詳細はREADME参照)。
    /// この関数は「次に新規接続する利用者がどのノードに割り当てられるか」
    /// だけを変える。
    pub fn rotate(&mut self, new_nodes: Vec<RelayNode>) {
        self.nodes = new_nodes;
    }

    /// 現在のプールに含まれるノードのエンドポイント集合(テスト・観測用)。
    pub fn endpoints(&self) -> HashSet<&str> {
        self.nodes.iter().map(|n| n.endpoint.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_apps_are_accepted_and_others_rejected() {
        assert!(is_app_allowed("open-tv-chat"));
        assert!(is_app_allowed("open-english"));
        assert!(is_app_allowed("aruaru-tokyo"));
        assert!(!is_app_allowed("https://example.com"));
        assert!(!is_app_allowed("some-random-website"));
        assert!(!is_app_allowed(""));
    }

    #[test]
    fn pick_returns_none_for_empty_pool() {
        let pool = NodePool::new(vec![]);
        assert!(pool.pick(0.5).is_none());
    }

    #[test]
    fn pick_selects_within_bounds_across_random_range() {
        let pool = NodePool::new(vec![
            RelayNode::new("hop1-a.example.internal"),
            RelayNode::new("hop1-b.example.internal"),
            RelayNode::new("hop1-c.example.internal"),
        ]);

        for i in 0..100 {
            let r = i as f64 / 100.0;
            let picked = pool.pick(r).expect("non-empty pool must return a node");
            assert!(pool.endpoints().contains(picked.endpoint.as_str()));
        }
    }

    #[test]
    fn rotate_replaces_pool_contents() {
        let mut pool = NodePool::new(vec![RelayNode::new("old-node.example.internal")]);
        assert_eq!(pool.len(), 1);

        pool.rotate(vec![
            RelayNode::new("new-node-a.example.internal"),
            RelayNode::new("new-node-b.example.internal"),
        ]);

        assert_eq!(pool.len(), 2);
        assert!(!pool.endpoints().contains("old-node.example.internal"));
        assert!(pool.endpoints().contains("new-node-a.example.internal"));
    }
}
