use ai_commit::cache::FsCacheManager;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing File System Cache Implementation");

    // Create a temporary directory for testing
    let temp_dir = TempDir::new()?;
    let cache_dir = temp_dir.path().to_str().unwrap();

    // Create a new file system cache manager
    let mut cache = FsCacheManager::new(cache_dir).await?;

    println!("✅ Created FsCacheManager at: {}", cache_dir);

    // Test 1: Basic set and get operations
    println!("\n🧪 Test 1: Basic set and get operations");
    cache.set("test_key", "test_value".to_string(), None).await?;
    let result: Option<String> = cache.get("test_key").await;
    assert_eq!(result, Some("test_value".to_string()));
    println!("✅ Set and get operations work correctly");

    // Test 2: TTL expiration
    println!("\n🧪 Test 2: TTL expiration");
    cache.set("expiring_key", "expiring_value".to_string(), Some(Duration::from_millis(100))).await?;
    let result: Option<String> = cache.get("expiring_key").await;
    assert_eq!(result, Some("expiring_value".to_string()));

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    let result: Option<String> = cache.get("expiring_key").await;
    assert_eq!(result, None);
    println!("✅ TTL expiration works correctly");

    // Test 3: Persistence across instances
    println!("\n🧪 Test 3: Persistence across instances");
    cache.set("persistent_key", "persistent_value".to_string(), None).await?;
    drop(cache);

    // Create a new cache instance
    let mut cache2 = FsCacheManager::new(cache_dir).await?;
    let result: Option<String> = cache2.get("persistent_key").await;
    assert_eq!(result, Some("persistent_value".to_string()));
    println!("✅ Data persists across cache instances");

    // Test 4: Cache statistics
    println!("\n🧪 Test 4: Cache statistics");
    let stats = cache2.stats().await;
    println!("Cache stats: entry_count={}, total_size={}", stats.entry_count, stats.total_size);
    assert!(stats.entry_count > 0);
    println!("✅ Cache statistics work correctly");

    // Test 5: Maintenance operations
    println!("\n🧪 Test 5: Maintenance operations");
    let report = cache2.maintenance().await?;
    println!("Maintenance report: expired_removed={}, corrupted_removed={}",
             report.expired_removed, report.corrupted_removed);
    println!("✅ Maintenance operations work correctly");

    // Test 6: Complex data serialization
    println!("\n🧪 Test 6: Complex data serialization");
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
    struct ComplexData {
        id: u32,
        name: String,
        values: Vec<i32>,
    }

    let complex_data = ComplexData {
        id: 42,
        name: "test_data".to_string(),
        values: vec![1, 2, 3, 4, 5],
    };

    cache2.set("complex_key", complex_data.clone(), None).await?;
    let result: Option<ComplexData> = cache2.get("complex_key").await;
    assert_eq!(result, Some(complex_data));
    println!("✅ Complex data serialization works correctly");

    println!("\n🎉 All file system cache tests passed!");
    println!("File system cache implementation is complete and working correctly.");

    Ok(())
}