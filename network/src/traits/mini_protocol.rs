pub trait MiniProtocol: Default {
    const NUMBER: u16;
    const READ_BUFFER_SIZE: usize;

    type States: Default;
}
