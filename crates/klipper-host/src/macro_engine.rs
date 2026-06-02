// crates/klipper-host/src/macro_engine.rs
use rhai::{Engine, AST, Scope};
use std::collections::HashMap;

pub struct HostMacroEngine {
    engine: Engine,
    compiled_macros: HashMap<String, AST>,
    gcode_sender: std::sync::mpsc::Sender<String>,
}

impl HostMacroEngine {
    pub fn new(gcode_sender: std::sync::mpsc::Sender<String>) -> Self {
        let mut engine = Engine::new();
        
        // Impose strict bounds on runtime execution to prevent thread starvation
        engine.set_max_operations(100_000);
        
        // Inject thread-safe G-Code execution command back into planner channel
        let sender_clone = gcode_sender.clone();
        engine.register_fn("gcode", move |cmd: String| {
            if let Err(e) = sender_clone.send(cmd) {
                eprintln!("Error sending G-Code macro command: {:?}", e);
            }
        });

        Self {
            engine,
            compiled_macros: HashMap::new(),
            gcode_sender,
        }
    }

    /// Compiles a G-Code macro script string into an optimized AST.
    pub fn register_macro(&mut self, name: &str, script: &str) -> Result<(), &'static str> {
        let ast = self.engine.compile(script).map_err(|_| "Compilation failure inside macro syntax")?;
        self.compiled_macros.insert(name.to_string(), ast);
        Ok(())
    }

    /// Executes a registered macro within a safe scope environment.
    pub fn execute_macro(&self, name: &str, params: Vec<rhai::Dynamic>) -> Result<(), &'static str> {
        let ast = self.compiled_macros.get(name).ok_or("Macro not registered")?;
        let mut scope = Scope::new();
        
        // Populate script parameters
        scope.push("params", params);
        
        self.engine.run_ast_with_scope(&mut scope, ast)
            .map_err(|_| "Runtime error occurred during macro execution")?;
        Ok(())
    }
}
