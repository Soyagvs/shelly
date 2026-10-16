use std::fs;
use std::path::PathBuf;

fn nvim_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(path)).expect("read Neovim runtime file")
}

#[test]
fn neovim_lsp_is_zero_config_and_attaches_on_shelly_filetype() {
    let plugin = nvim_file("editors/nvim/plugin/shelly-lsp.lua");
    let lsp = nvim_file("editors/nvim/lua/shelly/lsp.lua");

    assert!(plugin.contains("nvim_create_autocmd(\"FileType\""));
    assert!(plugin.contains("pattern = \"shelly\""));
    assert!(plugin.contains("require(\"shelly.lsp\").start(args.buf)"));
    assert!(lsp.contains("vim.fn.executable(\"shelly-lsp\")"));
    assert!(lsp.contains("vim.lsp.start({"));
    assert!(lsp.contains("cmd = { \"shelly-lsp\" }"));
    assert!(lsp.contains("vim.notify("));
}

#[test]
fn neovim_lsp_rejects_non_file_buffers_and_reuses_clients() {
    let lsp = nvim_file("editors/nvim/lua/shelly/lsp.lua");

    assert!(lsp.contains("vim.bo[bufnr].buftype ~= \"\""));
    assert!(lsp.contains("vim.api.nvim_buf_get_name(bufnr) == \"\""));
    assert!(lsp.contains("vim.lsp.get_clients"));
    assert!(lsp.contains("name = \"shelly-lsp\""));
}
