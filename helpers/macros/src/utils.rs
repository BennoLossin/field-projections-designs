use std::{
    io::{
        self,
        Write,
    },
    process::{
        Command,
        Stdio,
    },
};

pub(crate) fn rustfmt(
    txt: &str,
) -> io::Result<Result<String, (Option<i32>, String)>> {
    let mut cmd = Command::new("rustfmt");
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut fmt = cmd.spawn()?;
    fmt.stdin
        .as_mut()
        .expect("command to have stdin")
        .write_all(txt.as_bytes())?;
    let output = fmt.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Ok(Err((output.status.code(), err)));
    }
    let contents = String::from_utf8(output.stdout)
        .expect("rustfmt to have valid utf8 output")
        .trim()
        .to_string();
    Ok(Ok(contents))
}
