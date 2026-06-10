use rhai::{Engine, Scope, AST, Dynamic};
use std::collections::HashMap;
use std::sync::mpsc::{Sender, channel};

/// A sandboxed, hot-reloadable embedded scripting engine for G-Code macros.
pub struct HostMacroEngine {
    /// The Rhai engine instance, configured for safety and performance.
    engine: Engine,
    /// A thread-safe channel for sending G-Code commands back to the main pipeline.
    command_sender: Sender<String>,
    /// A map of pre-compiled macro ASTs for fast execution.
    macros: HashMap<String, AST>,
}

impl HostMacroEngine {
    /// Creates a new HostMacroEngine with strict safety limits and a G-Code callback.
    pub fn new() -> (Self, std::sync::mpsc::Receiver<String>) {
        let mut engine = Engine::new();

        // Set a strict operation limit to prevent infinite loops in macros.
        engine.set_max_operations(100_000);

        let (command_sender, command_receiver) = channel();
        let sender_clone = command_sender.clone();

        // Register a thread-safe G-Code callback function named "gcode".
        engine.register_fn("gcode", move |s: &str| {
            let _ = sender_clone.send(s.to_string());
        });

        (
            Self {
                engine,
                command_sender,
                macros: HashMap::new(),
            },
            command_receiver,
        )
    }

    /// Parses and saves a pre-compiled AST for a given macro script.
    ///
    /// Returns an error if the script fails to compile.
    pub fn register_macro(&mut self, name: &str, script: &str) -> Result<(), &'static str> {
        match self.engine.compile(script) {
            Ok(ast) => {
                self.macros.insert(name.to_string(), ast);
                Ok(())
            }
            Err(_) => Err("Failed to compile macro script"),
        }
    }

    /// Executes a pre-compiled macro by name within an isolated, local Scope.
    ///
    /// Returns an error if the macro is not found or if execution fails.
    pub fn execute_macro(&self, name: &str, params: Vec<Dynamic>) -> Result<(), String> {
        if let Some(ast) = self.macros.get(name) {
            let mut scope = Scope::new();

            // Note: In a real implementation, you might want to pass parameters
            // into the scope here. For this example, we're keeping it simple.

            self.engine
                .eval_ast_with_scope::<()>(&mut scope, ast)
                .map_err(|e| format!("Macro execution failed: {}", e))
        } else {
            Err(format!("Macro '{}' not found", name))
        }
    }
}
