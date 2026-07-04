use heapless::Vec;

const MAX_VARS: usize = 100;
const MAX_STACK_DEPTH: usize = 10;

pub struct MacroExecutor {
    variables: [f32; MAX_VARS],
    call_stack: Vec<usize, MAX_STACK_DEPTH>,
}

impl MacroExecutor {
    pub fn new() -> Self {
        Self {
            variables: [0.0; MAX_VARS],
            call_stack: Vec::new(),
        }
    }

    pub fn set_variable(&mut self, index: usize, value: f32) {
        if index < MAX_VARS {
            self.variables[index] = value;
        }
    }

    pub fn get_variable(&self, index: usize) -> f32 {
        if index < MAX_VARS {
            self.variables[index]
        } else {
            0.0
        }
    }

    // This is a simplified evaluator. A real implementation would parse expressions.
    pub fn evaluate_conditional(&self, var_index: usize, value: f32, op: &str) -> bool {
        let var = self.get_variable(var_index);
        match op {
            "GT" => var > value,
            "LT" => var < value,
            "EQ" => (var - value).abs() < f32::EPSILON,
            _ => false,
        }
    }
}
