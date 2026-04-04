use loro::*;

/// 操作履歴(OpLog)を、必要な場合にLazyLoadできるか？
///
/// まず、以下を仮定する。
/// - Server, PeerA, PeerB がいる
/// - Serverは常にFull Snapshotを保持する
/// - 全ての通信はServerを経由する
///     - あるPeerがした操作を他のPeerが知っている場合、その操作はServerも知っている
///
/// そのような状況で、
/// 1. PeerAがA1,A2,A3の操作をし、Serverに送信する
/// 2. PeerBはA1のみをServerから受信する
/// 3. PeerAはA3以前をShallow Snapshotでカットオフする
/// 4. PeerBはA1を起点に、B1を実行する
///
/// ここで、PeerAがB1を受け取ったとき、
/// a. PeerAは、B1がA1に依存していることを知ることができるか
///     - (Optional) A1がA3より過去であることを知ることはできる？
/// b. A3からA1に至る操作だけをServerから取得できるか
/// c. その後、B1を再適用し再度Syncできるか
/// を確認する
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

    // a. PeerAでB1をimportしてみる
    let result = peer_a.import(&b1);

    println!("\n====== a. PeerA imports B1 ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("\t{:?}", &result);
    println!("peer_b: {}", peer_b.get_text("text").to_string());

    // b. A3からA1に至る操作だけをServerから取得できるか
    let all_logs = server.export(ExportMode::all_updates()).unwrap();
    peer_a.import(&all_logs).unwrap();
    peer_a.import(&b1).unwrap();
    println!("\n====== a. PeerA imports B1 ======");
    println!("server: {}", server.get_text("text").to_string());
    println!("peer_a: {}", peer_a.get_text("text").to_string());
    println!("peer_b: {}", peer_b.get_text("text").to_string());
}
