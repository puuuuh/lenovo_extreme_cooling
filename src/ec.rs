use cpuio::Port;
use libc::ioperm;
use std::marker::PhantomData;

pub struct EmbeddedController {
    command: Port<u8>,
    data: Port<u8>,
    non_send: PhantomData<*const ()>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum EcCommand {
    Read = 0x80,
    Write = 0x81,
    #[allow(dead_code)]
    Query = 0x84,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum EcFlag {
    OutputBufferFull = 0,
    InputBufferFull = 1,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum EcPort {
    Command = 0x66,
    Data = 0x62,
}

impl EcPort {
    pub fn open(self) -> Result<Port<u8>, EmbeddedControllerError> {
        unsafe {
            let status = ioperm(self as u64, 1, 1);
            if status != 0 {
                Err(EmbeddedControllerError::IoPerm(status))
            } else {
                Ok(Port::new(self as u16))
            }
        }
    }
}

#[derive(Debug)]
pub enum EmbeddedControllerError {
    IoPerm(i32),
    Timeout,
}

impl EmbeddedController {
    pub fn new() -> Result<Self, EmbeddedControllerError> {
        Ok(Self {
            command: EcPort::Command.open()?,
            data: EcPort::Data.open()?,
            non_send: PhantomData::default(),
        })
    }

    fn wait_for(&mut self, flag: EcFlag, value: bool) -> Result<(), EmbeddedControllerError> {
        for _ in 0..20 {
            let val = (self.command.read() >> flag as u8 & 0x01) == 0x01;
            if val == value {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        return Err(EmbeddedControllerError::Timeout);
    }

    pub fn write(&mut self, port: u8, value: u8) -> Result<(), EmbeddedControllerError> {
        self.wait_for(EcFlag::InputBufferFull, false)?;
        self.command.write(EcCommand::Write as u8);
        self.wait_for(EcFlag::InputBufferFull, false)?;
        self.data.write(port);
        self.wait_for(EcFlag::InputBufferFull, false)?;
        self.data.write(value);
        self.wait_for(EcFlag::InputBufferFull, false)?;
        // Flush result value
        match self.wait_for(EcFlag::OutputBufferFull, true) {
            Ok(_) => {
                self.data.read();
            }
            Err(_) => {}
        };

        Ok(())
    }

    pub fn read(&mut self, port: u8) -> Result<u8, EmbeddedControllerError> {
        self.wait_for(EcFlag::InputBufferFull, false)?;
        self.command.write(EcCommand::Read as u8);
        self.wait_for(EcFlag::InputBufferFull, false)?;
        self.data.write(port);
        self.wait_for(EcFlag::OutputBufferFull, true)?;
        Ok(self.data.read())
    }
}
