# polysafe-gitfixer Default Configuration
#
# This configuration can be overridden by:
# 1. ~/.config/polysafe-gitfixer/config.ncl (user config)
# 2. .polysafe-gitfixer.ncl (project config)
# 3. Command-line arguments

let schema = import "schema.ncl" in

schema & {
  base_dir = "$HOME/workspace",

  backup_patterns = [
    "*-backup",
    "*.backup-*",
    "*_backup",
    "*.bak",
  ],

  max_scan_depth = 10,

  git_remotes = {
    preferred_remote = "origin",
    gitlab_orgs = [],
  },

  safety = {
    require_confirmation = true,
    dry_run = false,
    create_backups = true,
  },

  diff = {
    ignore_patterns = [
      ".git",
      "node_modules",
      "__pycache__",
      ".DS_Store",
      "*.pyc",
      ".venv",
      "target",
      "_build",
      "deps",
    ],
    max_file_size = 10485760,
    show_binary = false,
  },

  ui = {
    color = true,
    verbose = false,
    mode = 'Tui,
  },
}
