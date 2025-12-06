# polysafe-gitfixer Configuration Schema
#
# This Nickel schema defines the configuration structure for polysafe-gitfixer.
# Nickel provides compile-time type checking and validation.

{
  # Base directory to scan for git repositories
  base_dir
    | String
    | default = "$HOME/workspace"
    | doc "Root directory to scan for git repositories",

  # Patterns to identify backup directories
  backup_patterns
    | Array String
    | default = ["*-backup", "*.backup-*", "*_backup", "*.bak"]
    | doc "Glob patterns to identify backup directories",

  # Maximum depth to scan for repositories
  max_scan_depth
    | Number
    | default = 10
    | doc "Maximum directory depth when scanning for repos",

  # Git remote configuration
  git_remotes | {
    # GitHub username for matching repos
    github_user
      | String
      | optional
      | doc "GitHub username for remote matching",

    # GitLab organizations to check
    gitlab_orgs
      | Array String
      | default = []
      | doc "GitLab organizations to match against",

    # Preferred remote name (origin, upstream, etc.)
    preferred_remote
      | String
      | default = "origin"
      | doc "Preferred remote name to use for operations",
  },

  # Safety and confirmation settings
  safety | {
    # Require confirmation before destructive operations
    require_confirmation
      | Bool
      | default = true
      | doc "Require user confirmation before destructive operations",

    # Dry run mode - show what would happen without making changes
    dry_run
      | Bool
      | default = false
      | doc "Preview mode - no actual changes made",

    # Create backups before any modifications
    create_backups
      | Bool
      | default = true
      | doc "Create backups before modifying files",

    # Path to audit log
    audit_log_path
      | String
      | optional
      | doc "Path to append-only audit log (defaults to ~/.polysafe-gitfixer/audit.log)",
  },

  # Diff engine settings
  diff | {
    # Ignore patterns for diff comparison
    ignore_patterns
      | Array String
      | default = [".git", "node_modules", "__pycache__", ".DS_Store", "*.pyc"]
      | doc "Patterns to ignore during diff comparison",

    # Maximum file size to diff (in bytes)
    max_file_size
      | Number
      | default = 10485760  # 10MB
      | doc "Maximum file size for content comparison (larger files compared by hash)",

    # Show binary files in diff
    show_binary
      | Bool
      | default = false
      | doc "Include binary file changes in diff output",
  },

  # UI settings
  ui | {
    # Color output
    color
      | Bool
      | default = true
      | doc "Enable colored output",

    # Verbose mode
    verbose
      | Bool
      | default = false
      | doc "Show detailed progress information",

    # TUI or CLI mode
    mode
      | [| 'Tui, 'Cli |]
      | default = 'Tui
      | doc "Interface mode: TUI (interactive) or CLI (scripted)",
  },
}
