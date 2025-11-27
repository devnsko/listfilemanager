<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";

  interface MountPoint { path: string; label: string }
  interface FileEntry { path: string; relative_path: string; size: number }

  let mounts: MountPoint[] = [];
  let selectedMount = "";
  let files: FileEntry[] = [];
  let status = "";
  let filter = "";
  let moveTarget = "";
  let newFolderName = "";
  let filteredFiles: FileEntry[] = [];
  let darkMode = false;
  let showHidden = false;
  let filterInput: HTMLInputElement;

  // Helper to get filename and directory
  function splitPath(relativePath: string) {
    const parts = relativePath.split("/");
    const filename = parts.pop() || relativePath;
    const dir = parts.length > 0 ? parts.join("/") + "/" : "";
    return { filename, dir };
  }

  // Format file size
  function formatSize(bytes: number): string {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + " MB";
    return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GB";
  }

  function toggleTheme() {
    darkMode = !darkMode;
    localStorage.setItem("theme", darkMode ? "dark" : "light");
    document.body.className = darkMode ? 'dark' : '';
  }

  function toggleHidden() {
    showHidden = !showHidden;
    localStorage.setItem("showHidden", showHidden ? "true" : "false");
    if (selectedMount) {
      loadFiles();
    }
  }

  async function refreshMounts() {
    status = "Loading mounts...";
    try {
      mounts = await invoke<MountPoint[]>("list_candidate_mounts", {});
      status = `Found ${mounts.length} mount(s)`;
    } catch (e) {
      status = `Error: ${e}`;
    }
  }

  async function selectDirectoryDialog() {
    try {
      const dir = await open({ directory: true });
      if (dir) {
        selectedMount = Array.isArray(dir) ? dir[0] : dir;
        await loadFiles();
      }
    } catch (e) {
      status = `Dialog error: ${e}`;
    }
  }

  async function loadFiles() {
    if (!selectedMount) return;
    status = "Loading files...";
    try {
      files = await invoke<FileEntry[]>("list_files", { root: selectedMount, showHidden: showHidden });
      status = `Loaded ${files.length} files.`;
      applyFilter();
    } catch (e) {
      status = `Error: ${e}`;
    }
  }

  function applyFilter() {
    const f = filter.toLowerCase();
    filteredFiles = files.filter(file => !f || file.relative_path.toLowerCase().includes(f));
  }

  async function rename(file: FileEntry) {
    const currentName = file.relative_path.split("/").pop() || "";
    const newName = prompt("New file name", currentName);
    if (!newName || newName === currentName || newName.trim() === "") return;
    
    status = "Renaming...";
    try {
      await invoke("rename_file", { 
        root: selectedMount, 
        relativePath: file.relative_path, 
        newName: newName.trim() 
      });
      status = "Renamed successfully!";
      await loadFiles();
    } catch (e) {
      status = `Rename failed: ${e}`;
      alert("Rename failed: " + e);
    }
  }

  async function del(file: FileEntry) {
    if (!confirm(`Delete ${file.relative_path}?`)) return;
    
    status = "Deleting...";
    try {
      await invoke("delete_file", { 
        root: selectedMount, 
        relativePath: file.relative_path 
      });
      status = "Deleted successfully!";
      await loadFiles();
    } catch (e) {
      status = `Delete failed: ${e}`;
      alert("Delete failed: " + e);
    }
  }

  async function move(file: FileEntry) {
    let target = moveTarget;
    if (!target) {
      target = prompt("Move to folder (relative path, empty for root)", "") || "";
    }
    
    // Allow empty string to move to root
    if (target === null) return; // User cancelled
    
    target = target.trim();
    
    status = "Moving...";
    try {
      await invoke("move_file", { 
        root: selectedMount, 
        fromRelative: file.relative_path, 
        toRelativeDir: target, 
        createDir: true 
      });
      status = "Moved successfully!";
      moveTarget = ""; // Clear the move target after successful move
      await loadFiles();
    } catch (e) {
      status = `Move failed: ${e}`;
      alert("Move failed: " + e);
    }
  }

  async function createFolder() {
    if (!newFolderName || newFolderName.trim() === "") return;
    
    status = "Creating folder...";
    try {
      await invoke("create_folder", { 
        root: selectedMount, 
        relativeDir: newFolderName.trim() 
      });
      status = "Folder created successfully!";
      newFolderName = "";
      await loadFiles();
    } catch (e) {
      status = `Folder create failed: ${e}`;
      alert("Folder create failed: " + e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl/Cmd + R: Refresh
    if ((e.ctrlKey || e.metaKey) && e.key === "r") {
      e.preventDefault();
      refreshMounts();
    }
    // Ctrl/Cmd + O: Open directory
    if ((e.ctrlKey || e.metaKey) && e.key === "o") {
      e.preventDefault();
      selectDirectoryDialog();
    }
    // Ctrl/Cmd + F: Focus filter
    if ((e.ctrlKey || e.metaKey) && e.key === "f") {
      e.preventDefault();
      filterInput?.focus();
    }
    // Ctrl/Cmd + D: Toggle theme
    if ((e.ctrlKey || e.metaKey) && e.key === "d") {
      e.preventDefault();
      toggleTheme();
    }
    // Ctrl/Cmd + H: Toggle hidden files
    if ((e.ctrlKey || e.metaKey) && e.key === "h") {
      e.preventDefault();
      toggleHidden();
    }
  }

  onMount(() => {
    refreshMounts();
    const savedTheme = localStorage.getItem("theme");
    darkMode = savedTheme === "dark" || (!savedTheme && window.matchMedia("(prefers-color-scheme: dark)").matches);
    
    const savedShowHidden = localStorage.getItem("showHidden");
    showHidden = savedShowHidden === "true";
    
    // Apply theme immediately
    document.body.className = darkMode ? 'dark' : '';
    
    window.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleKeydown);
  });

  $: applyFilter();
  $: {
    if (typeof document !== 'undefined') {
      document.body.className = darkMode ? 'dark' : '';
    }
  }
</script>

<main class="container">
  <header>
    <h1>SD Cards Manager</h1>
    <div class="header-actions">
      <button class="theme-toggle" on:click={toggleHidden} title="Toggle hidden files (Ctrl+H)" class:active={showHidden}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          {#if showHidden}
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
          {:else}
            <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
            <line x1="1" y1="1" x2="23" y2="23"/>
          {/if}
        </svg>
      </button>
      <button class="theme-toggle" on:click={toggleTheme} title="Toggle theme (Ctrl+D)">
        {#if darkMode}
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="5"/>
            <line x1="12" y1="1" x2="12" y2="3"/>
            <line x1="12" y1="21" x2="12" y2="23"/>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
            <line x1="1" y1="12" x2="3" y2="12"/>
            <line x1="21" y1="12" x2="23" y2="12"/>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
          </svg>
        {:else}
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        {/if}
      </button>
    </div>
  </header>

  <div class="toolbar">
    <button on:click={refreshMounts} title="Refresh (Ctrl+R)">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="23 4 23 10 17 10"/>
        <polyline points="1 20 1 14 7 14"/>
        <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
      </svg>
      Refresh
    </button>
    <button on:click={selectDirectoryDialog} title="Open directory (Ctrl+O)">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      Open
    </button>
    <select bind:value={selectedMount} on:change={loadFiles}>
      <option value="">Select mount...</option>
      {#each mounts as m}
        <option value={m.path}>{m.label}</option>
      {/each}
    </select>
    <div class="search-box">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
      <input 
        bind:this={filterInput}
        placeholder="Filter files (Ctrl+F)" 
        bind:value={filter} 
      />
    </div>
  </div>

  {#if selectedMount}
    <div class="folder-tools">
      <input placeholder="New folder name" bind:value={newFolderName} />
      <button on:click={createFolder} disabled={!newFolderName}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        Create Folder
      </button>
      <input placeholder="Move target (empty = root, e.g. 'folder/subfolder')" bind:value={moveTarget} />
    </div>
    
    <div class="status-bar">
      <span>{status}</span>
      <span class="file-count">{filteredFiles.length} of {files.length} files</span>
    </div>

    <div class="file-list">
      {#each filteredFiles as f}
        {@const { filename, dir } = splitPath(f.relative_path)}
        <div class="file-item">
          <div class="file-info">
            <div class="file-name" title={f.relative_path}>{filename}</div>
            <div class="file-path" title={f.relative_path}>{dir || "/"}</div>
          </div>
          <div class="file-meta">
            <span class="file-size">{formatSize(f.size)}</span>
            <div class="file-actions">
              <button class="action-btn" on:click={() => rename(f)} title="Rename file">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                </svg>
              </button>
              <button class="action-btn" on:click={() => move(f)} title="Move file to folder">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="5 17 5 21 9 21"/>
                  <polyline points="19 7 19 3 15 3"/>
                  <line x1="5" y1="21" x2="14" y2="12"/>
                  <line x1="19" y1="3" x2="10" y2="12"/>
                </svg>
              </button>
              <button class="action-btn danger" on:click={() => del(f)} title="Delete file permanently">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6"/>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <svg class="empty-icon" width="80" height="80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
      </svg>
      <p>Select a mount or open a directory to get started</p>
      <p class="shortcuts">
        <kbd>Ctrl+O</kbd> to open · <kbd>Ctrl+R</kbd> to refresh · <kbd>Ctrl+H</kbd> hidden · <kbd>Ctrl+D</kbd> for theme
      </p>
    </div>
  {/if}
</main>

<!-- STYLES -->

<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  
  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    background: #fafafa;
    color: #202020;
    transition: background 0.3s ease, color 0.3s ease;
  }

  :global(body.dark) {
    background: #1a1a1a !important;
    color: #e0e0e0 !important;
  }

  main.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
    min-height: 100vh;
  }

  svg {
    flex-shrink: 0;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #e0e0e0;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }

  h1 {
    font-size: 1.5rem;
    font-weight: 600;
    color: #111;
  }

  .theme-toggle {
    background: transparent;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 0.5rem 0.75rem;
    font-size: 1.2rem;
    cursor: pointer;
    transition: all 0.2s ease;
    min-width: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .theme-toggle:hover {
    background: #f0f0f0;
    border-color: #ccc;
  }

  .theme-toggle.active {
    background: #e3f2fd;
    border-color: #4a9eff;
    color: #4a9eff;
  }

  .toolbar {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
    align-items: center;
  }

  button {
    background: white;
    border: 1px solid #ddd;
    border-radius: 6px;
    padding: 0.5rem 0.875rem;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s ease;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-weight: 500;
    white-space: nowrap;
  }

  button:hover:not(:disabled) {
    background: #f5f5f5;
    border-color: #bbb;
    transform: translateY(-1px);
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  }

  button:active:not(:disabled) {
    transform: translateY(0);
    box-shadow: none;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button svg {
    display: block;
  }

  select, input {
    background: white;
    border: 1px solid #ddd;
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    font-size: 0.875rem;
    transition: all 0.2s ease;
  }

  select {
    cursor: pointer;
    min-width: 200px;
    appearance: none;
    -webkit-appearance: none;
    padding-right: 2.25rem;
    padding-left: 0.75rem;
    transition: box-shadow 0.15s ease, border-color 0.15s ease;
  }

  input {
    min-width: 160px;
  }

  input:focus, select:focus {
    outline: none;
    border-color: #4a9eff;
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
  }

  .search-box {
    position: relative;
    flex: 1;
    max-width: 300px;
  }

  .search-icon {
    position: absolute;
    left: 0.75rem;
    top: 50%;
    transform: translateY(-50%);
    pointer-events: none;
    opacity: 0.5;
  }

  .search-box input {
    width: 100%;
    padding-left: 2.5rem;
  }

  .folder-tools {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .folder-tools input {
    flex: 1;
    min-width: 200px;
    max-width: 300px;
  }

  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    margin-bottom: 0.75rem;
    background: #f5f5f5;
    border-radius: 6px;
    font-size: 0.8rem;
    color: #666;
  }

  .file-count {
    font-weight: 500;
    color: #888;
  }

  .file-list {
    background: white;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
    overflow: hidden;
  }

  .file-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #f0f0f0;
    transition: all 0.15s ease;
  }

  .file-item:last-child {
    border-bottom: none;
  }

  .file-item:hover {
    background: #f8f9fa;
    transform: translateX(2px);
  }

  .file-info {
    flex: 1;
    min-width: 0;
    margin-right: 1rem;
  }

  .file-name {
    font-weight: 500;
    font-size: 0.9rem;
    color: #111;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file-path {
    font-size: 0.75rem;
    color: #888;
    font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 0.125rem;
  }

  .file-meta {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .file-size {
    font-size: 0.8rem;
    color: #666;
    font-weight: 500;
    min-width: 60px;
    text-align: right;
  }

  .file-actions {
    display: flex;
    gap: 0.25rem;
  }

  .action-btn {
    padding: 0.375rem 0.5rem;
    font-size: 1rem;
    border: 1px solid #e0e0e0;
    background: white;
    min-width: 0;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background: #f5f5f5;
    transform: scale(1.05);
  }

  .action-btn.danger {
    color: #d32f2f;
  }

  .action-btn.danger:hover {
    background: #ffebee;
    border-color: #ffcdd2;
  }

  .empty-state {
    text-align: center;
    padding: 4rem 2rem;
    color: #888;
  }

  .empty-icon {
    margin: 0 auto 1rem;
    opacity: 0.5;
  }

  .empty-state p {
    font-size: 1rem;
    margin-bottom: 0.5rem;
  }

  .shortcuts {
    font-size: 0.85rem !important;
    color: #aaa !important;
    margin-top: 1rem !important;
  }

  kbd {
    background: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 0.125rem 0.375rem;
    font-family: 'SF Mono', 'Monaco', 'Consolas', monospace;
    font-size: 0.75rem;
    font-weight: 600;
  }

  /* Dark mode */
  :global(body.dark) header {
    border-bottom-color: #333 !important;
  }

  :global(body.dark) h1 {
    color: #f0f0f0 !important;
  }

  :global(body.dark) .theme-toggle {
    background: transparent !important;
    border-color: #444 !important;
  }

  :global(body.dark) .theme-toggle:hover {
    background: #2a2a2a !important;
    border-color: #555 !important;
  }

  :global(body.dark) .theme-toggle.active {
    background: #1a3a52 !important;
    border-color: #4a9eff !important;
  }

  :global(body.dark) button {
    background: #2a2a2a !important;
    border-color: #444 !important;
    color: #e0e0e0 !important;
  }

  :global(body.dark) select {
    color: #e0e0e0;
    background-color: #2a2a2a;
    background-image:
      linear-gradient(45deg, transparent 50%, #e0e0e0 50%),
      linear-gradient(135deg, #e0e0e0 50%, transparent 50%);
    background-position: calc(100% - 1rem) calc(50% - 0.25rem), calc(100% - 0.7rem) calc(50% - 0.25rem);
    background-size: 0.45rem 0.45rem, 0.45rem 0.45rem;
    background-repeat: no-repeat;
    cursor: pointer;
    accent-color: #4a9eff;
  }

  :global(body.dark) select option {
    background: #2a2a2a;
    color: #e0e0e0;
  }

  /* Hide native IE arrow */
  :global(body.dark) select::-ms-expand {
    display: none;
  }

  /* Optional scrollbar styling for WebKit dropdowns */
  :global(body.dark) select::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  :global(body.dark) select::-webkit-scrollbar-thumb {
    background: #444;
    border-radius: 8px;
  }

  /* Inputs */
  :global(body.dark) input::placeholder {
    color: #9aa3ad !important;
    opacity: 1;
  }

  :global(body.dark) input {
    color: #e0e0e0;
    background-color: #2a2a2a;
    border-color: #444;
    caret-color: #4a9eff;
    transition: box-shadow 0.15s ease, border-color 0.15s ease;
  }

  :global(body.dark) input:focus,
  :global(body.dark) select:focus {
    box-shadow: 0 0 0 3px rgba(74,158,255,0.15) !important;
    border-color: #4a9eff !important;
  }

  /* Disabled state */
  :global(body.dark) input:disabled,
  :global(body.dark) select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  :global(body.dark) button:hover:not(:disabled) {
    background: #333 !important;
    border-color: #555 !important;
  }

  :global(body.dark) select,
  :global(body.dark) input {
    background: #2a2a2a !important;
    border-color: #444 !important;
    color: #e0e0e0 !important;
  }

  :global(body.dark) input:focus,
  :global(body.dark) select:focus {
    border-color: #4a9eff !important;
    box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.15) !important;
  }

  :global(body.dark) .status-bar {
    background: #252525 !important;
    color: #999 !important;
  }

  :global(body.dark) .file-count {
    color: #777 !important;
  }

  :global(body.dark) .file-list {
    background: #222 !important;
    border-color: #333 !important;
  }

  :global(body.dark) .file-item {
    border-bottom-color: #2a2a2a !important;
  }

  :global(body.dark) .file-item:hover {
    background: #282828 !important;
    transform: translateX(2px);
  }

  :global(body.dark) .file-name {
    color: #f0f0f0 !important;
  }

  :global(body.dark) .file-path {
    color: #777 !important;
  }

  :global(body.dark) .file-size {
    color: #999 !important;
  }

  :global(body.dark) .action-btn {
    background: #2a2a2a !important;
    border-color: #444 !important;
  }

  :global(body.dark) .action-btn:hover {
    background: #333 !important;
  }

  :global(body.dark) .action-btn.danger {
    color: #ff6b6b !important;
  }

  :global(body.dark) .action-btn.danger:hover {
    background: #3a1f1f !important;
    border-color: #5a2f2f !important;
  }

  :global(body.dark) .empty-state {
    color: #777 !important;
  }

  :global(body.dark) kbd {
    background: #2a2a2a !important;
    border-color: #444 !important;
    color: #aaa !important;
  }
</style>
