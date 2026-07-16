use rust_concurrency::rwlock::EducationalRwLock;

fn main() {
    let catalog = EducationalRwLock::new(vec!["mutex"]);

    catalog
        .with_read(|items| {
            println!("capítulos visibles: {}", items.len());
        })
        .unwrap();

    catalog.with_write(|items| items.push("rwlock")).unwrap();

    let snapshot = catalog.with_read(|items| items.join(", ")).unwrap();
    println!("catálogo actualizado: {snapshot}");
}
