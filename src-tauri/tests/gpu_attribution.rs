// story: e04s01
// scenario: SC-e04s01-P1-02
use sh_ui_lib::gpu::attribute_vram;

#[test]
fn gpu_attribution_counts_only_tree_pids() {
    // Vendor rows: pid, MiB — a pid may appear once per GPU it uses.
    let apps = [(1u32, 100u64), (2, 200), (999, 50), (1, 50)];
    let tree = [1u32, 2];

    let vram_mib = attribute_vram(&apps, &tree);

    // pid 1 on two GPUs (100 + 50) plus pid 2 (200); the foreign pid 999 never counts.
    assert_eq!(vram_mib, 350);
}

#[test]
fn gpu_attribution_of_an_empty_tree_is_zero_because_there_is_nothing_to_count() {
    let apps = [(7u32, 100u64)];
    let tree: [u32; 0] = [];
    assert_eq!(attribute_vram(&apps, &tree), 0);
}
