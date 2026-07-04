use heapless::{VecDeque, spsc::Producer};
use crate::hal::traits::Uart;
use crate::parser::{lexer, ast, modal::ModalState, dialect::{MachineDialect, MachineCommand}};

const LOOKAHEAD_BUFFER_SIZE: usize = 1024;

#[embassy_executor::task]
pub async fn gcode_streamer_task(
    mut uart: impl Uart,
    mut command_producer: Producer<'static, MachineCommand, LOOKAHEAD_BUFFER_SIZE>,
    dialect: impl MachineDialect,
) {
    let mut buffer: [u8; 128] = [0; 128];
    let mut modal_state = ModalState::default();
    let mut machine_state = super::dialect::MachineState { x: 0.0, y: 0.0, z: 0.0 };

    loop {
        if let Ok(len) = uart.read(&mut buffer).await {
            if len > 0 {
                if let Ok((_, tokens)) = lexer::lexer(core::str::from_utf8(&buffer[..len]).unwrap()) {
                    if let Ok(Some(ast_node)) = ast::build_ast(&tokens, &mut modal_state) {
                        if let Ok(command) = dialect.interpret(&ast_node, &mut machine_state) {
                            while command_producer.enqueue(command.clone()).is_err() {
                                embassy_time::Timer::after_millis(1).await; // Yield if buffer is full
                            }
                        }
                    }
                }
            }
        }
        embassy_time::Timer::after_millis(1).await; // Yield to executor
    }
}
