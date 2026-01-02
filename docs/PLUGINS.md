# Plugins

vibes supports native Rust plugins for extending functionality. Plugins can react to session events, track token usage, log conversations, and more.

## Using Plugins

```bash
# List installed plugins
vibes plugin list

# Enable/disable plugins
vibes plugin enable analytics
vibes plugin disable history

# Show plugin details
vibes plugin info my-plugin
```

## Plugin Directory

Plugins are installed to `~/.config/vibes/plugins/`:

```
~/.config/vibes/plugins/
├── registry.toml           # Tracks enabled plugins
└── my-plugin/
    ├── my-plugin.0.1.0.so  # Versioned binary
    ├── my-plugin.so        # Symlink to current version
    └── config.toml         # Plugin configuration
```

## Writing Plugins

See the [example plugin](../examples/plugins/hello-plugin/) for a complete working example.

```rust
use vibes_plugin_api::{export_plugin, Plugin, PluginContext, PluginError, PluginManifest};

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "my-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "My custom plugin".to_string(),
            ..Default::default()
        }
    }

    fn on_load(&mut self, ctx: &mut PluginContext) -> Result<(), PluginError> {
        ctx.log_info("Plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

export_plugin!(MyPlugin);
```

## Plugin API

The `vibes-plugin-api` crate provides the interface for plugin development. Plugins can:

- React to session lifecycle events (start, stop, pause, resume)
- Receive Claude Code hook events (tool use, message events)
- Register custom CLI subcommands under `vibes <plugin-name>`
- Register HTTP routes under `/api/plugins/<plugin-name>/`
- Access configuration with hot-reload support

---

## In-Tree Plugins

vibes ships with plugins developed in-tree under the `plugins/` directory.

### groove — Continual Learning

**groove** is the flagship vibes plugin—a continual learning system that captures what works in your AI coding sessions and injects those learnings into future sessions automatically.

> *"Find your coding groove"*

**Key features:**

- **Zero friction** — No annotation, no configuration. Just code and groove learns your patterns.
- **Scoped learnings** — Global, user, or project-level preferences
- **Multiple injection channels** — CLAUDE.md imports, SessionStart hooks, or per-prompt context
- **Confidence-based** — Learnings have confidence scores that grow with successful usage
- **Full control** — Pause, resume, forget, export, import

```bash
# Initialize groove for current project
vibes groove init

# Show learning status
vibes groove status

# List accumulated learnings
vibes groove insights

# Temporarily disable
vibes groove pause

# Export learnings for another machine
vibes groove export
```

**How it works:**

1. **Capture** — groove watches Claude Code sessions via hooks
2. **Extract** — Semantic analysis identifies patterns and preferences
3. **Store** — Learnings are persisted with confidence scores and provenance
4. **Inject** — Relevant learnings are inserted into future session contexts
5. **Assess** — Outcomes feed back to adjust confidence levels

See [groove branding guide](groove/BRANDING.md) for personality and design details.

**Status:** In active development (Phase 4 of vibes roadmap)

---

## Future Plugins

The vibes plugin architecture enables a rich ecosystem. Planned plugins include:

| Plugin | Description |
|--------|-------------|
| **analytics** | Token usage tracking, cost estimation, usage dashboards |
| **history** | Searchable conversation history with semantic search |
| **sync** | Cross-device learning synchronization |

Interested in building a plugin? Start with the [hello-plugin example](../examples/plugins/hello-plugin/).
