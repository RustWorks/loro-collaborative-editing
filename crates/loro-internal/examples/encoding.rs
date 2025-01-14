use std::time::Instant;

use bench_utils::TextAction;
use loro_internal::{loro::ExportMode, LoroDoc};

fn main() {
    let actions = bench_utils::get_automerge_actions();

    // let loro = LoroDoc::default();
    // let loro_b = LoroDoc::default();
    // let text = loro.get_text("text");
    // let mut i = 0;
    // let start = Instant::now();
    // for TextAction { pos, ins, del } in actions.iter() {
    //     {
    //         let mut txn = loro.txn().unwrap();
    //         text.delete(&mut txn, *pos, *del).unwrap();
    //         text.insert(&mut txn, *pos, ins).unwrap();
    //     }

    //     loro_b
    //         .import(&loro.export_from(&loro_b.oplog_vv()))
    //         .unwrap();
    //     i += 1;
    //     if i == 30000 {
    //         break;
    //     }
    // }

    // println!("{}ms", start.elapsed().as_millis());

    let loro = LoroDoc::default();
    let text = loro.get_text("text");

    for TextAction { pos, ins, del } in actions.iter() {
        let mut txn = loro.txn().unwrap();
        text.delete_with_txn(&mut txn, *pos, *del).unwrap();
        text.insert_with_txn(&mut txn, *pos, ins).unwrap();
        txn.commit().unwrap();
    }

    let start = Instant::now();
    let snapshot = loro.export_snapshot().unwrap();
    println!("Snapshot time {}ms", start.elapsed().as_millis());
    let output = miniz_oxide::deflate::compress_to_vec(&snapshot, 6);
    println!(
        "Snapshot+compression time {}ms",
        start.elapsed().as_millis()
    );

    println!(
        "snapshot size {} after compression {}",
        snapshot.len(),
        output.len(),
    );

    let start = Instant::now();
    let shallow_snapshot = loro
        .export(ExportMode::shallow_snapshot(&loro.oplog_frontiers()))
        .unwrap();
    println!("Shallow Snapshot time {}ms", start.elapsed().as_millis());
    let output = miniz_oxide::deflate::compress_to_vec(&shallow_snapshot, 6);
    println!(
        "Shallow Snapshot+compression time {}ms",
        start.elapsed().as_millis()
    );

    println!(
        "Shallow snapshot size {} after compression {}",
        shallow_snapshot.len(),
        output.len(),
    );

    let updates = loro.export_from(&Default::default());
    let output = miniz_oxide::deflate::compress_to_vec(&updates, 6);
    println!(
        "updates size {} after compression {}",
        updates.len(),
        output.len(),
    );

    let json_updates = serde_json::to_string(&loro.export_json_updates(
        &Default::default(),
        &loro.oplog_vv(),
        true,
    ))
    .unwrap();
    let output = miniz_oxide::deflate::compress_to_vec(json_updates.as_bytes(), 6);
    println!(
        "json updates size {} after compression {}",
        json_updates.len(),
        output.len(),
    );

    // {
    //     // Delta encoding

    {
        // Delta encoding

        let start = Instant::now();
        for _ in 0..10 {
            loro.export_from(&Default::default());
        }

        println!("Avg encode {}ms", start.elapsed().as_millis() as f64 / 10.0);

        let data = loro.export_from(&Default::default());
        let start = Instant::now();
        let n = 5;
        for _ in 0..n {
            let b = LoroDoc::default();
            b.detach();
            b.import(&data).unwrap();
        }

        println!(
            "Avg normal decode {}ms (without applying)",
            start.elapsed().as_millis() as f64 / (n as f64)
        );
        println!("size len={}", data.len());
        let d = miniz_oxide::deflate::compress_to_vec(&data, 10);
        println!("size after compress len={}", d.len());
    }

    // {
    //     // Snapshot encoding
    //     // println!("\n=======================\nSnapshot Encoding:");

    //     // let start = Instant::now();
    //     // for _ in 0..10 {
    //     //     loro.export_snapshot();
    //     // }

    //     // println!("Avg encode {}ms", start.elapsed().as_millis() as f64 / 10.0);

    //     // let data = loro.export_snapshot();
    //     // let start = Instant::now();
    //     // let times = 300;
    //     // for _ in 0..times {
    //     //     let b = LoroDoc::default();
    //     //     b.import(&data).unwrap();
    //     // }

    //     // println!(
    //     //     "Avg decode {}ms",
    //     //     start.elapsed().as_millis() as f64 / times as f64
    //     // );
    //     // println!("size len={}", data.len());
    //     // let d = miniz_oxide::deflate::compress_to_vec(&data, 10);
    //     // println!("size after compress len={}", d.len());
    // }
}
