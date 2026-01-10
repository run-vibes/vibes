# CLI Recordings

Terminal recordings for documentation and regression testing using [VHS](https://github.com/charmbracelet/vhs).

## Directory Structure

```
cli/recordings/
├── tapes/       # VHS tape files (scripts)
├── output/      # Generated GIFs (gitignored)
├── expected/    # Expected text output for regression
└── README.md    # This file
```

## Commands

```bash
# Generate all GIFs
just cli-record

# Verify CLI output matches expected
just cli-verify
```

## Creating New Recordings

1. Create a new `.tape` file in `tapes/`
2. Run `just cli-record` to generate the GIF
3. Capture expected output: `vibes <command> > expected/<name>.txt`
4. Commit both the tape and expected output
