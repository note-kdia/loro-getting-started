use loro::*;

/// 操作履歴(OpLog)を、必要な場合にLazyLoadできるか？
///
/// # 前提
///
/// - Server, PeerA, PeerB がいる
/// - Serverは常にFull Snapshotを保持する
/// - 全ての通信はServerを経由する
///     - あるPeerがした操作を他のPeerが知っている場合、その操作はServerも知っている
///
/// # シナリオ
///
/// 1. PeerAがA1,A2,A3の操作をし、Serverに送信する
/// 2. PeerBはA1のみをServerから受信する
/// 3. PeerAはA3以前をShallow Snapshotでカットオフする
/// 4. PeerBはA1を起点に、B1を実行する
/// 5. PeerAはすでにA4を実行している
///
/// ```text
/// Server: Hello
///          ↓
/// PeerA:  A1 → A2 → A3 [shallow cut] → A4
///          ↓
/// PeerB:  A1 → B1   ← B1はA1に依存、A2/A3を知らない（= A3とconcurrent）
/// ```
///
/// ここで、PeerAがB1を受け取ったとき、
/// a. PeerAは、B1がA1に依存していることを知ることができるか
/// b. A3からA1に至る操作だけをServerから取得できるか（Lazy Load）
/// c. その後、B1を再適用し再度Syncできるか
///
/// # 結果
///
/// | 検証項目 | 結果 | 理由 |
/// |---------|------|------|
/// | a. B1の依存検出 | **OK** | `Err(ImportUpdatesThatDependsOnOutdatedVersion)` で検出可能 |
/// | b. Lazy Load | **不可** | Loro の shallow snapshot の設計上の制約（後述） |
/// | c. Full re-sync後にB1適用 | **OK** | serverから `ExportMode::Snapshot` で新docを作れば正常にmerge |
///
/// # b. が不可能な理由
///
/// Loro ドキュメントより:
/// > "When using shallow snapshots, you cannot import updates that are
/// > concurrent to the snapshot's start version."
///
/// B1 は A1 の後に作られたが A2/A3 を知らない。つまり B1 は shallow frontier (A3) と
/// **concurrent（因果的に並行）** である。Loro はこのような更新の import を明示的にブロックする。
///
/// さらに、shallow snapshot は trimmed された歴史の ops を VersionVector には残しつつ
/// OpLog からは消去する。この「VV と OpLog の乖離」を修復する手段は提供されていない:
///
/// - `updates_in_range` で A1/A2 を取得しても、VV が「既知」と見なすため無視される
/// - server から `shallow_snapshot(at=A1)` を取得して歴史を後退延伸しようとしても同エラー
///
/// したがって、shallow snapshot 使用時に trimmed 領域と concurrent な更新を受け取った場合、
/// server から full snapshot で doc を再構築する（full re-sync）のが唯一の対処法となる。
///
fn main() {
    // Server, PeerA, PeerBがいる
    let server = LoroDoc::new();
    let peer_a = LoroDoc::new();
    let peer_b = LoroDoc::new();

    // 0. init state (synced)
    server.get_text("text").insert(0, "Hello").unwrap();
    let initial_state = server.export(ExportMode::Snapshot).unwrap();
    peer_a.import(&initial_state).unwrap();
    peer_b.import(&initial_state).unwrap();

    println!("\n====== 0. init ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // PeerA operate A1,A2,A3 and server knows
    let version_a0 = peer_a.oplog_vv();
    peer_a.get_text("text").insert(5, ", world").unwrap(); // Hello, world
    let a1 = peer_a.export(ExportMode::updates(&version_a0)).unwrap();
    server.import(&a1).unwrap();

    let version_a1 = peer_a.oplog_vv();
    peer_a.get_text("text").insert(12, "!").unwrap(); // Hello, world!
    let a2 = peer_a.export(ExportMode::updates(&version_a1)).unwrap();
    server.import(&a2).unwrap();

    let version_a2 = peer_a.oplog_vv();
    peer_a.get_text("text").insert(0, "✨️").unwrap(); // ✨️Hello, world!
    let a3 = peer_a.export(ExportMode::updates(&version_a2)).unwrap();
    server.import(&a3).unwrap();

    println!("\n====== 1. PeerA updates A1,A2,A3 ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // 2. PeerB receives A1 only
    peer_b.import(&a1).unwrap();

    println!("\n====== 2. PeerB receives A1 only ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // 3. PeerA cutoffs before A3
    let frontier_a3 = peer_a.state_frontiers();
    let shallow = peer_a
        .export(ExportMode::shallow_snapshot(&frontier_a3))
        .unwrap();
    let peer_a = LoroDoc::new();
    peer_a.import(&shallow).unwrap();

    println!("\n====== 3. PeerA cutoffs before A3 ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // 4. PeerB operates B1
    let version_a1 = peer_b.oplog_vv();
    peer_b.get_text("text").insert(8, "ired w").unwrap(); // Hello, wired world
    let b1 = peer_b.export(ExportMode::updates(&version_a1)).unwrap();
    server.import(&b1).unwrap();

    println!("\n====== 4. PeerB operates B1 ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // 5. PeerA operates B1
    let version_a3 = peer_a.oplog_vv();
    peer_a.get_text("text").insert(14, "🌍️").unwrap(); // ✨️Hello, world🌍️!
    let a4 = peer_a.export(ExportMode::updates(&version_a3)).unwrap();
    server.import(&a4).unwrap();

    println!("\n====== 5. PeerA operates A4 ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // a. PeerAは、B1がshallow frontier以前に依存していることを検出できるか？
    //    → Err(ImportUpdatesThatDependsOnOutdatedVersion) で検出可能
    let result = peer_a.import(&b1);

    println!("\n====== a. PeerA imports B1 ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("\t{:?}", &result);
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // b. A3からA1に至る操作だけをServerから取得できるか（Lazy Load）
    //    → 不可。以下の2つのアプローチを試したがどちらも失敗する。
    //
    //    試み1: updates_in_range で A1, A2 の ops だけを取得
    //      → peer_a の VV が A1/A2 を「既知」と見なすため import しても無視される
    //         (ImportStatus { success: VersionRange({}), pending: None })
    //
    //    試み2: shallow_snapshot(at=frontier_a1) で歴史を後退延伸
    //      → Err(ImportUpdatesThatDependsOnOutdatedVersion)
    //      → shallow frontier(A3) と concurrent な内容を含むため拒否される
    //
    //    根本原因: shallow snapshot は VV に trimmed ops を「既知」として残しつつ
    //    OpLog からは消去する。この乖離を修復する手段が Loro にはない。
    let frontier_a1 = server.vv_to_frontiers(&version_a1);

    println!("\n====== b. Attempt: extend history backwards via shallow_snapshot ======");
    println!(
        "peer_a shallow_since (A3): {:?}",
        peer_a.shallow_since_frontiers()
    );
    println!("frontier_a1:               {:?}", frontier_a1);

    let snapshot_from_a1 = server
        .export(ExportMode::shallow_snapshot(&frontier_a1))
        .unwrap();
    let result_b = peer_a.import(&snapshot_from_a1);
    println!("result: {:?}", result_b);
    println!("→ Loro は shallow frontier と concurrent な更新の import をブロックする");

    // c. full re-sync: shallow doc では B1 を適用できないため、
    //    server の full snapshot で新しい doc を再構築する（唯一の対処法）。
    //    server には A4 も送信済みなので、peer_a の未送信 ops がなければ欠損はない。
    println!("\n====== c. Full re-sync from server ======");
    let full_snapshot = server.export(ExportMode::Snapshot).unwrap();
    let peer_a_resynced = LoroDoc::new();
    peer_b.import_batch(&[a2, a3, a4]).unwrap();
    peer_a_resynced.import(&full_snapshot).unwrap();
    println!("server:          {}", server.get_text("text").to_string());
    println!(
        "peer_a_resynced: {}",
        peer_a_resynced.get_text("text").to_string()
    );
    println!("peer_b:          {}", peer_b.get_text("text").to_string());

    println!("\njson loro document for dubbging:");
    let updates = server.export_json_updates(&VersionVector::default(), &server.oplog_vv());
    println!("{}", serde_json::to_string(&updates).unwrap());
}
