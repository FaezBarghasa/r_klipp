use hal::uart::Uart;
use heapless::VecDeque;
use crate::lexer::{lexer, Token};
use crate::ast::{parse_line, AstNode};
use crate::dialect::{MachineDialect, MachineCommand};
use crate::modal::ModalState;

pub struct GcodeStreamer<UART, DIALECT>
where
    UART: Uart,
    DIALECT: MachineDialect,
{
    uart: UART,
    dialect: DIALECT,
    lookahead_buffer: VecDeque<MachineCommand, 1024>,
    modal_state: ModalState,
}

impl<UART, DIALECT> GcodeStreamer<UART, DIALECT>
where
    UART: Uart,
    DIALECT: MachineDialect,
{
    pub fn new(uart: UART, dialect: DIALECT) -> Self {
        Self {
            uart,
            dialect,
            lookahead_buffer: VecDeque::new(),
            modal_state: ModalState::default(),
        }
    }

    pub async fn run(&mut self) {
        let mut buffer = [0u8; 256];
        loop {
            if self.lookahead_buffer.is_full() {
                embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
                continue;
            }

            let bytes_read = self.uart.read(&mut buffer).await.unwrap();
            if bytes_read > 0 {
                let mut input = &buffer[..bytes_read];
                while !input.is_empty() {
                    match lexer(input) {
                        Ok((remaining, tokens)) => {
                            if let Ok(Some(ast_node)) = parse_line(&tokens, &mut self.modal_state) {
                                if let Ok(machine_command) = self.dialect.interpret(&ast_node, &mut self.modal_state) {
                                    self.lookahead_buffer.push_back(machine_command).unwrap();
                                }
                            }
                            input = remaining;
                        }
                        Err(_) => {
                            // Handle lexer error
                            break;
                        }
                    }
                }
            }
        }
    }
}
