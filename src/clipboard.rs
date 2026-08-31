use super::*;

struct Tool {
    copy: &'static [&'static str],
    // Read only by the unix timed-clear path.
    #[cfg_attr(not(unix), allow(dead_code))]
    paste: Option<&'static [&'static str]>,
}

const WAYLAND: Tool = Tool {
    copy: &["wl-copy"],
    paste: Some(&["wl-paste", "--no-newline"]),
};
const XCLIP: Tool = Tool {
    copy: &["xclip", "-selection", "clipboard"],
    paste: Some(&["xclip", "-selection", "clipboard", "-o"]),
};
const XSEL: Tool = Tool {
    copy: &["xsel", "--clipboard", "--input"],
    paste: Some(&["xsel", "--clipboard", "--output"]),
};
const PBCOPY: Tool = Tool {
    copy: &["pbcopy"],
    paste: Some(&["pbpaste"]),
};
const CLIP_EXE: Tool = Tool {
    copy: &["clip"],
    paste: None,
};

fn tool() -> Result<Tool> {
    let candidates = if env::var_os("WAYLAND_DISPLAY").is_some() {
        [WAYLAND, XCLIP, XSEL, PBCOPY, CLIP_EXE]
    } else {
        [XCLIP, XSEL, WAYLAND, PBCOPY, CLIP_EXE]
    };
    candidates
        .into_iter()
        .find(|tool| in_path(tool.copy[0]))
        .ok_or_else(|| {
            anyhow!("no clipboard tool found (looked for wl-copy, xclip, xsel, pbcopy, clip)")
        })
}

fn in_path(name: &str) -> bool {
    env::var_os("PATH").is_some_and(|path| {
        env::split_paths(&path).any(|dir| {
            let candidate = dir.join(name);
            candidate.is_file() || candidate.with_extension("exe").is_file()
        })
    })
}

/// Copy `text` and schedule a best-effort clear after `seconds`. The clearer
/// is a detached shell that receives the expected value on stdin — never
/// argv — and skips the clear when the clipboard has since changed (when a
/// paste tool exists to check with). Clipboard-history managers are outside
/// anyone's reach; `clip` warns about that once at the call site.
pub(crate) fn copy(text: &str, seconds: u64) -> Result {
    let tool = tool()?;

    let mut child = process::Command::new(tool.copy[0])
        .args(&tool.copy[1..])
        .stdin(process::Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to run `{}`", tool.copy[0]))?;
    child
        .stdin
        .as_mut()
        .expect("stdin was piped")
        .write_all(text.as_bytes())?;
    let status = child.wait()?;
    ensure!(status.success(), "`{}` failed: {status}", tool.copy[0]);

    #[cfg(unix)]
    schedule_clear(&tool, text, seconds)?;
    #[cfg(not(unix))]
    if seconds > 0 {
        eprintln!("note: timed clipboard clear is not implemented on this platform");
    }

    Ok(())
}

#[cfg(unix)]
fn schedule_clear(tool: &Tool, text: &str, seconds: u64) -> Result {
    if seconds == 0 {
        return Ok(());
    }

    let copy = tool.copy.join(" ");
    let script = match tool.paste {
        Some(paste) => format!(
            "expected=$(cat); sleep {seconds}; \
             [ \"$({})\" = \"$expected\" ] && printf '' | {copy}",
            paste.join(" ")
        ),
        None => format!("cat > /dev/null; sleep {seconds}; printf '' | {copy}"),
    };

    let mut child = process::Command::new("sh")
        .args(["-c", &script])
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
        .context("failed to spawn the clipboard clearer")?;
    child
        .stdin
        .as_mut()
        .expect("stdin was piped")
        .write_all(text.as_bytes())?;
    // Deliberately not waited on: it outlives this process by design.
    Ok(())
}
