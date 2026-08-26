use anyhow::Result;
use minijinja::{Environment, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Template pre-rendering engine matching Moonraker's Jinja2 environment.
#[derive(Clone)]
pub struct TemplateEngine {
    env: Arc<Environment<'static>>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Custom global secrets.get(key)
        env.add_function("secrets_get", |_key: String| -> Value {
            Value::from("")
        });

        Self { env: Arc::new(env) }
    }

    /// Render a raw config or macro string through the template engine.
    pub fn render_string(&self, template_str: &str, context: &HashMap<String, serde_json::Value>) -> Result<String> {
        let minijinja_ctx = minijinja::Value::from_serialize(context);
        let rendered = self.env.render_str(template_str, minijinja_ctx)?;
        Ok(rendered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jinja_rendering() {
        let engine = TemplateEngine::new();
        let mut ctx = HashMap::new();
        ctx.insert("printer_name".to_string(), serde_json::json!("Voron2.4"));

        let template = "Printer: {{ printer_name }}";
        let rendered = engine.render_string(template, &ctx).unwrap();
        assert_eq!(rendered, "Printer: Voron2.4");
    }
}
