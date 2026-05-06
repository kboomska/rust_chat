// aaaa\0bbbbbb
// 04aaaa\06bbbbbb

#[derive(Clone)]
pub struct Message {
    pub sender: String,
    pub text: String,
}

pub fn write_str(
    string: &str,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    out.write_all(&(string.len() as u32).to_le_bytes())?;
    out.write_all(string.as_bytes())?;
    Ok(())
}

pub fn write_message(
    msg: Message,
    out: &mut impl std::io::Write,
) -> std::io::Result<()> {
    write_str(&msg.sender, out)?;
    write_str(&msg.text, out)?;
    Ok(())
}

pub fn read_message(from: &mut impl std::io::Read) -> std::io::Result<Message> {
    let sender = read_str(from)?;
    let text = read_str(from)?;
    Ok(Message { sender, text })
}

pub fn read_str(from: &mut impl std::io::Read) -> std::io::Result<String> {
    let mut buf = [0u8; std::mem::size_of::<u32>()];
    from.read_exact(&mut buf)?;
    let len = u32::from_le_bytes(buf);

    let mut buf = vec![0u8; len as usize];
    from.read_exact(&mut buf)?;

    let string = String::from_utf8(buf).map_err(|err| {
        log::error!("Error decoding message from UTF-8: {:?}", err);
        std::io::ErrorKind::InvalidData
    })?;
    Ok(string)
}
