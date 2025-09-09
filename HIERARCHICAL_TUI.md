# Hierarchical TUI Implementation

## Overview
A new hierarchical Terminal User Interface (TUI) has been implemented to provide a more logical and intuitive navigation flow for Git repository browsing. This implementation follows a clear hierarchy: Main Menu → Category Selection → Commit List → Diff View.

## Command
```bash
ai-commit --tui
```

## Navigation Structure

### 1. Main Menu (Entry Point)
The main menu serves as the central hub with the following options:
- **Branches**: View and navigate all local and remote branches
- **Tags**: Browse repository tags with version information
- **Remotes**: View configured remote repositories
- **Current Branch Log**: Quick access to current branch commits
- **Query History**: View past Git queries and their results

### 2. Navigation Flow
```
Main Menu
├── Branches → Branch List → Commit List → Diff View
├── Tags → Tag List → Commit List → Diff View
├── Remotes → Remote List → (Info View)
├── Current Branch Log → Commit List → Diff View
└── Query History → Query List → (Results View)
```

### 3. Key Bindings

#### Global Navigation
- `j` or `↓`: Move down in lists
- `k` or `↑`: Move up in lists
- `Enter`: Select/Enter deeper level
- `ESC` or `Backspace`: Go back to previous level
- `q`: Quit (from main menu) or go back (from sub-views)

#### In Diff View
- `j`/`k` or `Tab`/`Shift+Tab`: Navigate between files
- `J`/`K`: Scroll diff content up/down
- `f`/`b` or `PageDown`/`PageUp`: Page through diff
- `v`: Cycle through view modes (Unified → Side-by-Side → Split)
- `1`: Unified view mode
- `2`: Side-by-Side view mode
- `3`: Split view mode
- `t`: Toggle file list panel
- `h`: Toggle syntax highlighting
- `g`: Jump to first file
- `G`: Jump to last file

### 4. Features

#### Breadcrumb Navigation
- Shows current location in the hierarchy
- Example: `Repository Overview > Branches > feature/new-ui > Diff (abc12345)`

#### Context-Aware Status Bar
- Displays relevant shortcuts based on current view
- Shows item counts and selection position
- Indicates current view mode in diff viewer

#### Smart Data Loading
- Lazy loading of commit histories
- Async Git operations for responsive UI
- Cached data for improved performance

### 5. View Types

#### Main Menu View
- Clean list of available categories
- Shows item counts for each category
- Highlighted selection with arrow indicator

#### Branch/Tag List View
- Color-coded entries (current branch in green, remote in blue)
- Shows branch/tag names with additional metadata
- Supports scrolling for long lists

#### Commit List View
- Displays commit hash, message, author, and timestamp
- Color-coded by commit type (feat, fix, docs, etc.)
- Scrollable with selection highlighting

#### Diff View
- Professional diff viewer with multiple display modes
- File list sidebar (toggleable)
- Syntax highlighting for code changes
- Line numbers and change statistics

### 6. Implementation Details

#### File Structure
- `/src/tui_hierarchical.rs`: Main implementation file
- `/src/diff_viewer.rs`: Diff viewing functionality (enhanced with Clone trait)
- `/src/query_history.rs`: Query history management

#### Key Components
- `ViewStack`: Manages navigation history and breadcrumb trail
- `ViewContext`: Stores state for each view level
- `ViewType`: Enum defining all possible view types
- `App`: Main application state and event handling

#### Architecture Highlights
- Async/await for non-blocking Git operations
- Event-driven keyboard input handling
- Modular view rendering system
- Memory-efficient data structures

### 7. Comparison with Existing TUIs

| Feature | --query-tui | --query-tui-pro | --tui (New) |
|---------|------------|-----------------|-------------|
| Navigation | Flat | Tab-based | Hierarchical |
| Breadcrumbs | No | No | Yes |
| Logical Flow | Basic | Good | Excellent |
| Return Navigation | Limited | Tab switching | ESC to go back |
| Diff Viewer | Basic | Integrated | Professional |
| User Experience | Simple | Feature-rich | Intuitive |

### 8. Future Enhancements
- Search functionality within views
- Bookmark frequently accessed branches/tags
- Customizable color schemes
- Export diff to file
- Integration with external diff tools

## Testing
To test the new hierarchical TUI:
```bash
# Build the project
cargo build --release

# Run the hierarchical TUI
./target/release/ai-commit --tui

# Navigate using keyboard
# - Enter to go deeper
# - ESC to go back
# - j/k to move up/down
# - q to quit from main menu
```

## Migration Notes
The new hierarchical TUI (`--tui`) is designed to eventually replace both `--query-tui` and `--query-tui-pro` commands, providing a unified and more intuitive interface for Git repository browsing.