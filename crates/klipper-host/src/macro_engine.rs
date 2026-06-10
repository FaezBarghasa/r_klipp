use rhai::{Dynamic, Engine, Scope, AST};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

/// A sandboxed interpreter for executing G-Code macros via Rhai.
pub struct HostMacroEngine {
    engine: Engine,
    command_pipeline: Sender<String>,
    asts: HashMap<String, AST>,
}

impl HostMacroEngine {
    /// Initializes a new macro engine enforcing strict safety limitations.
    pub fn new(command_pipeline: Sender<String>) -> Self {
        let mut engine = Engine::new();

        // Enforce execution limits to prevent infinite loops locking the parser
        engine.set_max_operations(100_000);

        // Share the sender safely across threading contexts for use inside the scripting environment closure
        let tx_shared = Arc::new(Mutex::new(command_pipeline.clone()));

        // Register a thread-safe callback mapping named "gcode" to inject strings into the pipeline
        engine.register_fn("gcode", move |cmd: &str| {
            if let Ok(sender) = tx_shared.lock() {
                // Ignore send errors in case the pipeline receiver is already disconnected
                let _ = sender.send(cmd.to_string());
            }
        });

        Self {
            engine,
            command_pipeline,
            asts: HashMap::new(),
        }
    }

    /// Compiles a script into an Abstract Syntax Tree (AST) and caches it in memory.
    pub fn register_macro(&mut self, name: &str, script: &str) -> Result<(), &'static str> {
        let ast = self
            .engine
            .compile(script)
            .map_err(|_| "Failed to compile macro script into AST")?;
        
        self.asts.insert(name.to_string(), ast);
        Ok(())
    }

    /// Evaluates a pre-compiled macro AST within an isolated local scope.
    pub fn execute_macro(&self, name: &str, params: Vec<Dynamic>) -> Result<(), &'static str> {
        let ast = self.asts.get(name).ok_or("Macro not found")?;
        
        let mut scope = Scope::new();
        
        // Inject parameters dynamically into the isolated scope block
        for (index, param) in params.into_iter().enumerate() {
            scope.push(format!("param_{}", index), param);
        }

        self.engine
            .eval_ast_with_scope::<()>(&mut scope, ast)
            .map_err(|_| "Failed to execute macro AST")?;

        Ok(())
    }
}