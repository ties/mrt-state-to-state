use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

/// Downloads a file from a URL and saves it to the specified path
/// Files are saved as-is (compressed files remain compressed)
fn download_file(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading {} to {:?}", url, dest);

    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to download {}: HTTP {}", url, response.status()).into());
    }

    let content = response.bytes()?;
    let mut file = fs::File::create(dest)?;
    file.write_all(&content)?;

    println!("Downloaded and saved to {:?} ({} bytes)", dest, content.len());
    Ok(())
}

/// Helper function to get a cache directory for test data
fn get_cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Downloads or retrieves from cache an MRT file
fn get_mrt_file(url: &str, filename: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = get_cache_dir()?;
    let file_path = cache_dir.join(filename);

    if file_path.exists() {
        println!("Using cached file: {:?}", file_path);
        return Ok(file_path);
    }

    download_file(url, &file_path)?;
    Ok(file_path)
}

#[test]
#[ignore] // This test downloads files from the internet, so it's marked as ignored by default
fn test_parse_ripe_mrt_bview() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger for debugging
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    // Download the bview file (initial BGP table state) - kept as .gz
    let bview_url = "https://data.ris.ripe.net/rrc18/2023.05/bview.20230501.0000.gz";
    let bview_path = get_mrt_file(bview_url, "bview.20230501.0000.gz")?;

    println!("Testing with bview file: {:?}", bview_path);

    // Create a processor and load the initial state
    // bgpkit-parser will automatically decompress the .gz file
    let mut processor = mrt_state_to_state::mrt_processor::MrtProcessor::default();
    processor.process_bview(&bview_path)?;

    // Get the current state and verify we have some peers
    let state = processor.get_current_state();
    assert!(!state.is_empty(), "Should have parsed at least one BGP peer from bview file");

    println!("Successfully parsed bview file with {} peers", state.len());

    // Print some statistics
    let total_prefixes: usize = state.values()
        .map(|peer_state| {
            // We can't directly access prefix_announcements, so we'll just count peers
            1
        })
        .sum();

    println!("Total peers with data: {}", total_prefixes);

    Ok(())
}

#[test]
#[ignore] // This test downloads files from the internet, so it's marked as ignored by default
fn test_parse_ripe_mrt_updates() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger for debugging
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    // Download the bview file (initial BGP table state) - kept as .gz
    let bview_url = "https://data.ris.ripe.net/rrc18/2023.05/bview.20230501.0000.gz";
    let bview_path = get_mrt_file(bview_url, "bview.20230501.0000.gz")?;

    // Download a few update files - kept as .gz
    let update_urls = vec![
        "https://data.ris.ripe.net/rrc18/2023.05/updates.20230501.0000.gz",
        "https://data.ris.ripe.net/rrc18/2023.05/updates.20230501.0005.gz",
        "https://data.ris.ripe.net/rrc18/2023.05/updates.20230501.0010.gz",
    ];

    let mut update_paths = Vec::new();
    for url in &update_urls {
        let filename = url.split('/').last().unwrap();
        let path = get_mrt_file(url, filename)?;
        update_paths.push(path);
    }

    println!("Testing with bview file: {:?}", bview_path);
    println!("Testing with {} update files", update_paths.len());

    // Create a processor and load the initial state
    // bgpkit-parser will automatically decompress the .gz files
    let mut processor = mrt_state_to_state::mrt_processor::MrtProcessor::new(180, Some(3));
    processor.process_bview(&bview_path)?;

    let initial_peer_count = processor.get_current_state().len();
    println!("Initial peer count from bview: {}", initial_peer_count);

    // Process each update file
    for (i, update_path) in update_paths.iter().enumerate() {
        println!("Processing update file {} of {}: {:?}", i + 1, update_paths.len(), update_path);
        processor.process_update_file(update_path)?;

        let current_peer_count = processor.get_current_state().len();
        println!("Peer count after update {}: {}", i + 1, current_peer_count);
    }

    // Verify we still have peers after processing updates
    let final_state = processor.get_current_state();
    assert!(!final_state.is_empty(), "Should still have BGP peers after processing updates");

    println!("Successfully processed all files. Final peer count: {}", final_state.len());

    Ok(())
}

#[test]
fn test_processor_basic_functionality() {
    // Test that we can create a processor with different configurations
    let processor1 = mrt_state_to_state::mrt_processor::MrtProcessor::new(180, None);
    assert_eq!(processor1.get_current_state().len(), 0);

    let processor2 = mrt_state_to_state::mrt_processor::MrtProcessor::new(180, Some(3));
    assert_eq!(processor2.get_current_state().len(), 0);

    let processor3 = mrt_state_to_state::mrt_processor::MrtProcessor::default();
    assert_eq!(processor3.get_current_state().len(), 0);
}

#[test]
#[ignore]
fn test_parse_single_update_file() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger for debugging
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();

    // Download just one update file for a quick test - kept as .gz
    let update_url = "https://data.ris.ripe.net/rrc18/2023.05/updates.20230501.0000.gz";
    let filename = "updates.20230501.0000.gz";
    let update_path = get_mrt_file(update_url, filename)?;

    println!("Testing with single update file: {:?}", update_path);

    // Create a processor (without initial bview, to test processing updates standalone)
    // bgpkit-parser will automatically decompress the .gz file
    let mut processor = mrt_state_to_state::mrt_processor::MrtProcessor::default();
    processor.process_update_file(&update_path)?;

    // Get the current state
    let state = processor.get_current_state();
    println!("Parsed {} peers from single update file", state.len());

    // Verify we have at least some peers
    assert!(!state.is_empty(), "Should have parsed at least one BGP peer from update file");

    Ok(())
}
